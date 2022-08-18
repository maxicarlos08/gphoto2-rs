// This will only work for Nikon DSLR cameras
use gphoto2::{camera::CameraEvent, widget::WidgetValue, Context, Result};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
  let context = Context::new()?;
  let camera = context.autodetect_camera()?;

  let mut shutter_speed = camera.config_key("shutterspeed")?;
  let mut bulb_setting = camera.config_key("bulb")?;

  shutter_speed.set_value(WidgetValue::Menu("Bulb".to_string()))?;
  camera.set_config(&shutter_speed)?;

  println!("Starting bulb capture");

  bulb_setting.set_value(WidgetValue::Toggle(true))?;
  camera.set_config(&bulb_setting)?;

  sleep(Duration::from_secs(2));

  bulb_setting.set_value(WidgetValue::Toggle(false))?;
  camera.set_config(&bulb_setting)?;

  let mut retry = 0;

  loop {
    let event = camera.wait_event(Duration::from_secs(10))?;

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
