use gphoto2::{Context, Result};

fn main() -> Result<()> {
  let cameras = Context::new()?.list_cameras()?;

  println!("Available cameras:\n{:#?}", cameras.to_vec());

  Ok(())
}
