use gphoto2::{Context, Result};

fn main() -> Result<()> {
  let context = Context::new()?;
  let cameras = context.list_cameras()?;

  println!("Available cameras:\n{:#?}", cameras.to_vec()?);

  Ok(())
}
