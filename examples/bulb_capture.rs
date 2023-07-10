//! This is an example of using gphoto action widgets
//!
//! There are also actions for capturing movies, changing liveview, etc.  
//! This example will only work for Nikon DSLR cameras.

mod logging;

use gphoto2::widget::{RadioWidget, ToggleWidget};
use gphoto2::{camera::CameraEvent, Context, Result};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
  logging::setup();

  let camera = Context::new()?.autodetect_camera().wait()?;

  let shutter_speed = camera.config_key::<RadioWidget>("shutterspeed").wait()?;
  let bulb_setting = camera.config_key::<ToggleWidget>("bulb").wait()?;

  shutter_speed.set_choice("Bulb")?;
  camera.set_config(&shutter_speed).wait()?;

  println!("Starting bulb capture");

  bulb_setting.set_toggled(true);
  camera.set_config(&bulb_setting).wait()?;

  sleep(Duration::from_secs(2));

  bulb_setting.set_toggled(false);
  camera.set_config(&bulb_setting).wait()?;

  let mut retry = 0;

  loop {
    let event = camera.wait_event(Duration::from_secs(10)).wait()?;

    if let CameraEvent::NewFile(file) = event {
      println!("New file: {}", file.name());
      // To download the file using file.download(&camera, path)

      break;
    }

    retry += 1;

    println!("Retry: Received other event {:?}", event);

    if retry > 10 {
      println!("No new file added :(");
      break;
    }
  }

  println!("Bulb capture done");

  Ok(())
}
