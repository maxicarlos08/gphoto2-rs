//! Allows thread safe interaction with libgphoto2

use crate::{
  context::CancelHandler,
  thread::{ThreadManager, THREAD_MANAGER},
  Context,
};
use std::{
  future::Future,
  ops::Deref,
  sync::{Arc, RwLock},
  task::{Poll, Waker},
};

/// Handy macro to create a new task
///
/// # CAUTION
///
/// Any closure passed will be marked as [`Send`] (to allow sending raw pointer).
///
/// Creating a task inside a task will cause a deadlock.
macro_rules! task {
  { context: $context:expr, exec: $task:tt } => {
    $crate::task::Task::new(Box::new(move || $task), $context)
  };
}

pub(crate) use task;

type ToBeRunTask<T> = Option<(Box<dyn FnOnce() -> T + 'static>, oneshot::Sender<UnsafeSend<T>>)>;

/// Allows awaiting (or blocking) libgphoto2 function responses
pub struct Task<T> {
  rx: oneshot::Receiver<UnsafeSend<T>>,
  cancel: Arc<RwLock<bool>>,
  waker: Arc<RwLock<Option<Waker>>>,
  waker_set: bool,
  task: ToBeRunTask<T>,
  context: Option<Context>,
}

struct TaskCancelHandler(Arc<RwLock<bool>>);

/// Marks any value as [`Send`]
struct UnsafeSend<T>(pub T);

impl<T> Task<T>
where
  T: 'static,
{
  /// Starts a new task
  ///
  /// # CAUTION
  ///
  /// Any closure passed here will be marked as [`Send`]
  pub(crate) fn new(fun: Box<dyn FnOnce() -> T>, context: Option<&Context>) -> Self
  where
    T: 'static,
  {
    ThreadManager::ensure_started();

    let (tx, rx) = oneshot::channel();

    Self {
      rx,
      cancel: Arc::new(RwLock::new(false)),
      waker: Arc::new(RwLock::new(None::<Waker>)),
      waker_set: false,
      task: Some((fun, tx)),
      context: context.map(ToOwned::to_owned),
    }
  }

  fn start_task(&mut self) {
    if let Some((fun, tx)) = self.task.take() {
      let fun = UnsafeSend(fun);
      let waker_clone = self.waker.clone();
      let context = UnsafeSend(self.context.take()); // Take the context and move it to the task function
      let cancel = self.cancel.clone();

      #[allow(unused_must_use)]
      let task = Box::new(move || {
        // Set context cancel function
        if let Some(mut context) = context.take() {
          let cancel_handler = TaskCancelHandler(cancel);
          context.set_cancel_handler(cancel_handler);
        }

        tx.send(UnsafeSend(fun.call()));
        if let Some(waker) = waker_clone.write().map(|mut guard| guard.take()).ok().flatten() {
          waker.wake()
        }
      });

      if let Some(manager) = THREAD_MANAGER.read().unwrap().as_ref() {
        manager.spawn_task(task);
      }
    }
  }

  /// Block until the response if available
  pub fn wait(mut self) -> T {
    self.start_task();
    self.rx.recv().unwrap().0 // TODO: Check if this .unwrap is OK
  }

  /// Request the current task to be cancelled
  pub fn cancel(&self) {
    if let Ok(mut cancel) = self.cancel.write() {
      *cancel = true;
    }
  }
}

impl<T> Future for Task<T>
where
  T: 'static,
{
  type Output = T;

  fn poll(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    if !self.waker_set {
      if let Ok(mut waker) = self.waker.write() {
        *waker = Some(cx.waker().clone());
      }
      self.waker_set = true;
    }

    self.start_task();

    if let Ok(value) = self.rx.try_recv() {
      Poll::Ready(value.0)
    } else {
      Poll::Pending
    }
  }
}

impl CancelHandler for TaskCancelHandler {
  fn cancel(&mut self) -> bool {
    matches!(self.0.read().map(|cancel| *cancel), Ok(true))
  }
}

impl<T> Deref for UnsafeSend<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> UnsafeSend<T> {
  fn take(self) -> T {
    self.0
  }
}

impl<T> UnsafeSend<Box<dyn FnOnce() -> T>>
where
  T: 'static,
{
  fn call(self) -> T {
    self.0()
  }
}

unsafe impl<T> Send for UnsafeSend<T> {}
