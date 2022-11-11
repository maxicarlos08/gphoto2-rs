#![allow(dead_code)] // This is just an example

use gphoto2::{Context, Result};
use std::{collections::HashMap, path::Path};

struct ContextProgress {
  message: String,
  target: f32,
  current: f32,
}

struct ProgressManager {
  progresses: HashMap<u32, ContextProgress>,
  next_progress_id: u32,
}

impl gphoto2::context::ProgressHandler for ProgressManager {
  fn start(&mut self, target: f32, message: String) -> u32 {
    let id = self.next_progress_id;

    self.next_progress_id += 1;
    self.progresses.insert(id, ContextProgress { message, target, current: 0.0 });
    self.on_progress_update();
    id
  }

  fn update(&mut self, id: u32, progress: f32) {
    if let Some(cprogress) = self.progresses.get_mut(&id) {
      cprogress.current = progress;
    }
    self.on_progress_update();
  }

  fn stop(&mut self, id: u32) {
    self.progresses.remove(&id);
    self.on_progress_update()
  }
}

impl ProgressManager {
  fn new() -> Self {
    Self { progresses: Default::default(), next_progress_id: 0 }
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

  context.set_progress_handlers(ProgressManager::new());

  let camera = context.autodetect_camera().wait()?;
  let image = camera.capture_image().wait()?;
  camera
    .fs()
    .download_to(&image.folder(), &image.name(), Path::new(&image.name().into_owned()))
    .wait()?;

  Ok(())
}
