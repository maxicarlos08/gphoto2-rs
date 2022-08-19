//! Capture a preview and save it to /tmp/preview_image

use gphoto2::{Context, Result};
use std::{fs, io::Write};

fn main() -> Result<()> {
  let context = Context::new()?;
  let camera = context.autodetect_camera()?;

  let mut file = fs::File::create("/tmp/preview_image")?;

  let preview = camera.capture_preview()?;
  let data = preview.get_data()?;
  println!("Data size: {}", data.len());

  file.write_all(&data)?;

  Ok(())
}
