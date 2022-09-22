//! Recursively list all configuration
//! Warning: Output might be very large

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  let camera = Context::new()?.autodetect_camera()?;
  println!("{:#?}", camera.config()?);
  Ok(())
}
