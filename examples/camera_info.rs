use gphoto2::{Context, Result};

fn main() -> Result<()> {
  env_logger::init();

  let camera = Context::new()?.autodetect_camera()?;

  println!("==== SUMMARY   ====\n{}", camera.summary()?);
  println!("==== ABILITIES ====\n{:#?}", camera.abilities());
  println!("==== STORAGES  ====");

  let storages = camera.storages()?;

  for (i, storage) in storages.iter().enumerate() {
    println!("---- Storage #{} ----\n{:#?}", i, storage);
  }

  println!("==== PORT      ====\n{:#?}", camera.port_info()?);

  Ok(())
}
