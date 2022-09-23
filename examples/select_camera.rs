//! Select a camera by name
//! Usage: select_camera <camera_name>
//! To get a list of connected cameras, run example list_cameras

use gphoto2::{Context, Result};

fn main() -> Result<()> {
  let camera_name = std::env::args().nth(1).expect("Missing argument: camera_model");

  let context = Context::new()?;

  let camera_list = context.list_cameras()?;
  let camera_list = camera_list.to_vec();

  if let Some((camera, port)) = camera_list.iter().find(|(name, _)| name == &camera_name) {
    let _camera = context.get_camera(&camera.to_owned(), &port.to_owned())?;

    println!("Found camera {}!", camera_name);
  } else {
    Err(format!("Could not find camera with name '{}'", camera_name).as_str())?;
  }

  Ok(())
}
