//! Recursively list all configuration
//! Warning: Output might be very large

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  env_logger::init();

  let camera = Context::new()?.autodetect_camera().wait()?;
  println!("{:#?}", camera.config().wait()?);
  Ok(())
}
