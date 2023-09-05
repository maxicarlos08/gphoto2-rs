//! Test if the camera can be dropped while there are is still eg. a widget of that camera present

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let camera = Context::new()?.autodetect_camera().wait()?;

  let widget = camera.config().wait()?;
  let abilities = camera.abilities();

  drop(camera);

  widget.children_count();
  println!("{:?}", abilities);

  Ok(())
}
