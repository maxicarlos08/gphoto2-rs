//! Select a camera by name
//! Usage: select_camera <camera_name>
//! To get a list of connected cameras, run example list_cameras

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  env_logger::init();

  let camera_name = std::env::args().nth(1).expect("Missing argument: camera_model");

  let context = Context::new()?;

  let camera_desc = context
    .list_cameras().wait()?
    .find(|desc| desc.model == camera_name)
    .ok_or_else(|| format!("Could not find camera with name '{}'", camera_name))?;

  let _camera = context.get_camera(&camera_desc).wait()?;

  Ok(())
}
