use gphoto2::list::CameraDescriptor;
use gphoto2::{Context, Result};

fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let context = Context::new()?;

  println!("Available cameras:");
  for CameraDescriptor { model, port } in context.list_cameras().wait()? {
    println!("  {} on port {}", model, port);
  }

  Ok(())
}
