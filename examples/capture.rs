mod logging;

use gphoto2::{Context, Result};
use std::path::Path;

fn main() -> Result<()> {
  logging::setup();

  let camera = Context::new()?.autodetect_camera().wait()?;

  let file = camera.capture_image().wait()?;
  println!("Captured image {}", file.name());

  camera
    .fs()
    .download_to(&file.folder(), &file.name(), Path::new(&file.name().to_string()))
    .wait()?;
  println!("Downloaded image {}", file.name());

  Ok(())
}
