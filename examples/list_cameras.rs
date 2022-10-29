use gphoto2::list::CameraDescriptor;
use gphoto2::{Context, Result};

fn main() -> Result<()> {
  env_logger::init();

  let context = Context::new().wait()?;

  println!("Available cameras:");
  for CameraDescriptor { model, port } in context.list_cameras()? {
    println!("  {} on port {}", model, port);
  }

  Ok(())
}
