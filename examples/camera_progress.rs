#![allow(dead_code)] // This is just an example

use gphoto2::{Context, Result};
use std::{
  collections::HashMap,
  path::Path,
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
    self.on_progress_update();
    id
  }

  fn update_progress(&mut self, id: u32, progress: f32) {
    self.progresses.get_mut(&id).map(|cprogress| cprogress.current = progress);
    self.on_progress_update();
  }

  fn end_progress(&mut self, id: u32) {
    self.progresses.remove(&id);
    self.on_progress_update()
  }

  fn on_progress_update(&self) {
    println!("Current number of running progresses: {}", self.progresses.len());
    for (id, progress) in self.progresses.iter() {
      println!("   - progress #{:03}: [{}] {:0.1}% done", id, progress.message, progress.progress())
    }
  }
}

impl ContextProgress {
  /// Convenience function Converts the progress to 0..1
  fn progress(&self) -> f32 {
    self.current / self.target
  }
}

fn main() -> Result<()> {
  let mut context = Context::new()?;

  env_logger::init();

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

  let camera = context.autodetect_camera()?;
  let image = camera.capture_image()?;
  camera.fs().download_to(&image.folder(), &image.name(), Path::new(&image.name().into_owned()))?;

  Ok(())
}
