use gphoto2::{Context, Result};

fn main() -> Result<()> {
  let context = Context::new()?;

  let cameras = context.list_cameras().unwrap();

  println!("Available cameras: {:?}", cameras.to_vec()?);

  Ok(())
}