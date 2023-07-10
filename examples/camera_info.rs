mod logging;

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  logging::setup();

  let camera = Context::new()?.autodetect_camera().wait()?;

  println!("==== SUMMARY   ====\n{}", camera.summary()?);
  println!("==== ABILITIES ====\n{:#?}", camera.abilities());
  println!("==== STORAGES  ====");

  let storages = camera.storages().wait()?;

  for (i, storage) in storages.iter().enumerate() {
    println!("---- Storage #{} ----\n{:#?}", i, storage);
  }

  println!("==== PORT      ====\n{:#?}", camera.port_info()?);

  Ok(())
}
