//! Recursively list all configuration
//! Warning: Output might be very large

mod logging;

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  logging::setup();

  let camera = Context::new()?.autodetect_camera().wait()?;
  println!("{:#?}", camera.config().wait()?);
  Ok(())
}
