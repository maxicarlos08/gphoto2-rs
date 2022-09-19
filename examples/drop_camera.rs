//! Test if the camera can be dropped while there are is still eg. a widget of that camera present

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  let context = Context::new()?;
  let camera = context.autodetect_camera()?;

  let widget = camera.config()?;
  let something = camera.abilities()?;

  drop(camera);

  widget.children_count()?;
  println!("{:?}", something);

  Ok(())
}
