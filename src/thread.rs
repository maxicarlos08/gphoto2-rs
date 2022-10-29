use std::{
  collections::VecDeque,
  sync::{Arc, Mutex, Once, RwLock},
  thread,
  thread::JoinHandle,
};

pub static THREAD_MANAGER: RwLock<Option<ThreadManager>> = RwLock::new(None);

pub type TaskFunc = Box<dyn FnOnce() + Send>;

pub struct ThreadManager {
  handle: JoinHandle<()>,
  queue: Arc<Mutex<VecDeque<TaskFunc>>>,
}

pub struct UnsafeSend<T>(T);

impl ThreadManager {
  pub fn ensure_started() {
    static START: Once = Once::new();

    START.call_once(|| *THREAD_MANAGER.write().unwrap() = Some(ThreadManager::new().unwrap()));
  }

  fn new() -> Result<Self, std::io::Error> {
    let queue = Arc::new(Mutex::new(VecDeque::new()));

    let queue_clone = queue.clone();
    let thread_handle = thread::Builder::new()
      .name("gphoto2".to_string()) // Give the thread a name for debugging
      .spawn(move || start_thread(queue_clone))?;

    Ok(Self { handle: thread_handle, queue })
  }

  fn continue_camera_thread(&self) {
    self.handle.thread().unpark();
  }

  pub fn spawn_task(&self, task: TaskFunc) {
    self.queue.lock().unwrap().push_back(task);
    self.continue_camera_thread();
  }
}

fn start_thread(queue: Arc<Mutex<VecDeque<TaskFunc>>>) {
  loop {
    let tasks = queue.lock().unwrap().drain(..).collect::<Vec<TaskFunc>>();

    for task in tasks {
      task();
    }

    thread::park();
  }
}

unsafe impl<T> Send for UnsafeSend<T> {}
