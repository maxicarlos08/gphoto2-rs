use gphoto2::{Context, Result};
use std::path::Path;

fn main() -> Result<()> {
  let context = Context::new()?;
  let camera = context.autodetect_camera()?;

  let file = camera.capture_image()?;
  println!("Captured image {}", file.name());

  file.download(&camera, Path::new(&file.name().to_string()))?;
  println!("Downloaded image {}", file.name());

  Ok(())
}
