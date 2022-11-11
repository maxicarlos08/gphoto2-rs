use std::{
  sync::{Once, RwLock},
  thread,
  thread::JoinHandle,
};

use crossbeam_channel::{unbounded, Receiver, Sender};

pub static THREAD_MANAGER: RwLock<Option<ThreadManager>> = RwLock::new(None);

pub type TaskFunc = Box<dyn FnOnce() + Send>;

pub struct ThreadManager {
  _handle: JoinHandle<()>,
  send_task: Sender<TaskFunc>,
}

impl ThreadManager {
  pub fn ensure_started() {
    static START: Once = Once::new();

    START.call_once(|| *THREAD_MANAGER.write().unwrap() = Some(ThreadManager::new().unwrap()));
  }

  fn new() -> Result<Self, std::io::Error> {
    let (send_task, receive_task) = unbounded();

    let thread_handle = thread::Builder::new()
      .name("gphoto2".to_string()) // Give the thread a name for debugging
      .spawn(move || start_thread(receive_task))?;

    Ok(Self { _handle: thread_handle, send_task })
  }

  #[allow(unused_must_use)]
  pub fn spawn_task(&self, task: TaskFunc) {
    self.send_task.send(task);
  }
}

fn start_thread(recv_task: Receiver<TaskFunc>) {
  while let Ok(fun) = recv_task.recv() {
    fun()
  }
}
