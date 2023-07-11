//! Recursively list all configuration
//! Warning: Output might be very large

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let camera = Context::new()?.autodetect_camera().wait()?;
  println!("{:#?}", camera.config().wait()?);
  Ok(())
}
