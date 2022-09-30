#![allow(dead_code)] // This is just an example

use gphoto2::{Context, Result};
use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

struct ContextProgress {
  message: String,
  target: f32,
  current: f32,
}

struct ProgressManager {
  progresses: HashMap<u32, ContextProgress>,
  next_progress_id: u32,
}

impl ProgressManager {
  fn new() -> Self {
    Self { progresses: Default::default(), next_progress_id: 0 }
  }

  fn new_progress(&mut self, message: String, target: f32) -> u32 {
    let id = self.next_progress_id;
    self.next_progress_id += 1;
    self.progresses.insert(id, ContextProgress { message: message, target: target, current: 0.0 });
    id
  }

  fn update_progress(&mut self, id: u32, progress: f32) {
    self.progresses.get_mut(&id).map(|cprogress| cprogress.current = progress);
  }

  fn end_progress(&mut self, id: u32) {
    self.progresses.remove(&id);
  }

  // When actually using this you would an another function which updates your progress bars and call that function
  // in new_progress, update_progress and end_progress
}

impl ContextProgress {
  /// Convenience function Converts the progress to 0..1
  fn progress(&self) -> f32 {
    self.current / self.target
  }
}

fn main() -> Result<()> {
  let mut context = Context::new()?;

  // Wrapping this in an Arc is necessary because the context functions may outlive the context wrapper's scope
  // (not in this case though because we are in `fn main()` and the context is dropped when the program ends)
  let progresses = Arc::new(RwLock::new(ProgressManager::new()));

  context.set_progress_functions(
    {
      let progresses_ref = progresses.clone();
      Box::new(move |target, message| progresses_ref.write().unwrap().new_progress(message, target))
    },
    {
      let progresses_ref = progresses.clone();
      Box::new(move |id, current| {
        progresses_ref.write().unwrap().update_progress(id, current);
      })
    },
    {
      let progresses_ref = progresses.clone();
      Box::new(move |id| progresses_ref.write().unwrap().end_progress(id))
    },
  );

  Ok(())
}
