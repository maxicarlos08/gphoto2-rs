//! Recursively list all configuration
//! Warning: Output might be very large

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  env_logger::init();

  let camera = Context::new().wait()?.autodetect_camera()?;
  println!("{:#?}", camera.config()?);
  Ok(())
}
