//! Allows thread safe interaction with libgphoto2

use crate::thread::{ThreadManager, THREAD_MANAGER};
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
  { $(context: $context:expr,)? exec: $task:tt } => {
    $crate::task::Task::new(Box::new(move || $task))
  };
}

pub(crate) use task;

/// Allows awaiting (or blocking) libgphoto2 function responses
pub struct Task<T> {
  rx: oneshot::Receiver<UnsafeSend<T>>,
  // TODO: Implement libgphoto2 task cancelling
  cancel: Arc<RwLock<bool>>,
  waker: Arc<RwLock<Option<Waker>>>,
  waker_set: bool,
}

/// Marks any value as [`Send`]
struct UnsafeSend<T>(pub T);

impl<T> Task<T> {
  /// Starts a new task
  ///
  /// # CAUTION
  ///
  /// Any closure passed here will be marked as [`Send`]
  pub(crate) fn new(f: Box<dyn FnOnce() -> T + 'static>) -> Self
  where
    T: 'static,
  {
    ThreadManager::ensure_started();

    let (tx, rx) = oneshot::channel();

    let waker = Arc::new(RwLock::new(None::<Waker>));
    let cancel = Arc::new(RwLock::new(false));
    let waker_clone = waker.clone();
    let _cancel_clone = cancel.clone();

    let f = UnsafeSend(f);

    #[allow(unused_must_use)]
    let task = Box::new(move || {
      tx.send(UnsafeSend(f.call()));
      if let Some(waker) = waker_clone.write().unwrap().take() {
        waker.wake()
      }
    });

    if let Some(manager) = THREAD_MANAGER.read().unwrap().as_ref() {
      manager.spawn_task(task);
    }

    Self { rx, cancel, waker, waker_set: false }
  }

  /// Block until the response if available
  pub fn wait(self) -> T {
    self.rx.recv().unwrap().0 // TODO: Check if this .unwrap is OK
  }

  /// Request the current task to be cancelled
  pub fn cancel(&self) {
    if let Ok(mut cancel) = self.cancel.write() {
      *cancel = true;
    }
  }
}

impl<T> Future for Task<T> {
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

    if let Ok(value) = self.rx.try_recv() {
      Poll::Ready(value.0)
    } else {
      Poll::Pending
    }
  }
}

impl<T> Deref for UnsafeSend<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    &self.0
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
