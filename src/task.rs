//! Allows thread safe interaction with libgphoto2

use crate::thread::{ThreadManager, THREAD_MANAGER};
use std::{
  future::Future,
  sync::{Arc, RwLock},
  task::{Poll, Waker},
};

/// Allows awaiting (or blocking) libgphoto2 function responses
pub struct Task<T> {
  rx: oneshot::Receiver<T>,
  // TODO: Implement libgphoto2 task cancelling
  cancel: Arc<RwLock<bool>>,
  waker: Arc<RwLock<Option<Waker>>>,
  waker_set: bool,
}

impl<T> Task<T> {
  pub(crate) fn new(f: Box<dyn FnOnce() -> T + Send + 'static>) -> Self
  where
    T: Send + 'static,
  {
    ThreadManager::ensure_started();

    let (tx, rx) = oneshot::channel();

    let waker = Arc::new(RwLock::new(None::<Waker>));
    let cancel = Arc::new(RwLock::new(false));
    let waker_clone = waker.clone();
    let _cancel_clone = cancel.clone();

    let task = Box::new(move || {
      tx.send(f()).unwrap();
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
  pub fn wait(self) -> Result<T, oneshot::RecvError> {
    self.rx.recv()
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
      Poll::Ready(value)
    } else {
      Poll::Pending
    }
  }
}
