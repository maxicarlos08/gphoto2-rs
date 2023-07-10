//! Capture a preview and save it to /tmp/preview_image

mod logging;

use gphoto2::{Context, Result};
use std::{fs, io::Write};

fn main() -> Result<()> {
  logging::setup();

  let context = Context::new()?;
  let camera = context.autodetect_camera().wait()?;

  let mut file = fs::File::create("/tmp/preview_image")?;

  let preview = camera.capture_preview().wait()?;
  let data = preview.get_data(&context).wait()?;
  println!("Data size: {}", data.len());

  file.write_all(&data)?;

  Ok(())
}
