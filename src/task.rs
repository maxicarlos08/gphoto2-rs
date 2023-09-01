//! Allows thread safe interaction with libgphoto2

use crate::{
  context::{CancelHandler, ProgressHandler},
  thread::{TaskFunc, ThreadManager, THREAD_MANAGER},
  Context,
};
use crossbeam_channel::{bounded, Receiver, RecvError, Sender};
use std::{
  future::Future,
  ops::Deref,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  task::{Poll, Waker},
};

type ToBeRunTask<T> = Option<(Box<dyn FnOnce() -> T + Send>, Sender<T>)>;

#[derive(Clone, Copy)]
pub(crate) struct BackgroundPtr<T>(pub *mut T);

/// Allows awaiting (or blocking) libgphoto2 function responses
pub struct Task<T> {
  rx: Receiver<T>,
  cancel: Arc<AtomicBool>,
  set_waker: Sender<Waker>,
  waker_set: bool,
  task: ToBeRunTask<T>,
  context: Option<BackgroundPtr<libgphoto2_sys::GPContext>>,
  progress_handler: Option<Box<dyn ProgressHandler>>,
  recv_waker: Option<Receiver<Waker>>,
}

struct TaskCancelHandler(Arc<AtomicBool>);

impl<T> Task<T>
where
  T: 'static + Send,
{
  /// Starts a new task
  pub(crate) unsafe fn new(fun: impl FnOnce() -> T + 'static + Send) -> Self {
    ThreadManager::ensure_started();

    let (tx, rx) = bounded(1);
    let (tx_waker, rx_waker) = bounded(1);

    Self {
      rx,
      cancel: Arc::new(AtomicBool::new(false)),
      set_waker: tx_waker,
      recv_waker: Some(rx_waker),
      waker_set: false,
      task: Some((Box::new(fun), tx)),
      context: None,
      progress_handler: None,
    }
  }

  pub(crate) fn context(mut self, context: BackgroundPtr<libgphoto2_sys::GPContext>) -> Self {
    self.context = Some(context);

    self
  }

  fn start_task(&mut self) {
    if let Some((fun, tx)) = self.task.take() {
      let mut opt_context_ptr = self.context.take();
      let recv_waker = self.recv_waker.take();
      let progress_handler = self.progress_handler.take();
      let cancel = self.cancel.clone();

      #[allow(unused_must_use)]
      let task: TaskFunc = Box::new(move || {
        let mut context = None;

        if let Some(context_ptr) = opt_context_ptr.as_mut() {
          let mut task_context = Context::from_ptr(*context_ptr);

          let cancel_handler = TaskCancelHandler(cancel);
          task_context.set_cancel_handler(cancel_handler);

          if let Some(progress_handler) = progress_handler {
            task_context.set_progress_handlers(progress_handler)
          }

          context = Some(task_context);
        }

        let result = fun();

        if let Some(context) = context.as_mut() {
          context.unset_cancel_handlers();
          context.unset_progress_handlers();
        }

        tx.send(result);
        if let Some(waker) = recv_waker.and_then(|w| w.try_recv().ok()) {
          waker.wake();
        }
      });

      if let Some(manager) = THREAD_MANAGER.read().unwrap().as_ref() {
        manager.spawn_task(task);
      }
    }
  }

  /// Block until the response if available
  pub fn wait(self) -> T {
    self.try_wait().unwrap()
  }

  /// Try blocking until a result is available
  pub fn try_wait(mut self) -> Result<T, RecvError> {
    self.start_task();
    self.rx.recv()
  }

  /// Set the progress handler for the task
  ///
  /// Must be called before the task is started
  pub fn set_progress_handler<H>(&mut self, handler: H)
  where
    H: ProgressHandler,
  {
    self.progress_handler = Some(Box::new(handler));
  }

  /// Set the progress handler for the task
  ///
  /// Must be called before the task is started
  pub fn with_progress_handler<H>(mut self, handler: H) -> Self
  where
    H: ProgressHandler,
  {
    self.progress_handler = Some(Box::new(handler));
    self
  }

  /// Request the current task to be cancelled
  pub fn cancel(&self) {
    self.cancel.store(true, Ordering::Relaxed);
  }

  /// Starts the task in background
  pub(crate) fn background(&mut self) {
    self.start_task();
  }
}

impl<T> Future for Task<T>
where
  T: 'static + Send,
{
  type Output = T;

  fn poll(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    #[allow(unused_must_use)]
    if !self.waker_set {
      self.set_waker.send(cx.waker().clone());
      self.waker_set = true;
    }

    self.start_task();

    if let Ok(value) = self.rx.try_recv() {
      Poll::Ready(value)
    } else {
      Poll::Pending
    }
  }
}

impl CancelHandler for TaskCancelHandler {
  fn cancel(&mut self) -> bool {
    self.0.load(Ordering::Relaxed)
  }
}

impl<T> Deref for BackgroundPtr<T> {
  type Target = *mut T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

unsafe impl<T> Send for BackgroundPtr<T> {}
unsafe impl<T> Sync for BackgroundPtr<T> {}
impl<T> Unpin for Task<T> {}
