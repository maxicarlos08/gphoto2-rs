use gphoto2::{camera::CameraEvent, Context, Result};
use std::time::Duration;

fn main() -> Result<()> {
  let camera = Context::new()?.autodetect_camera()?;
  camera.abilities();
  // camera.summary(); <- This outputs events in an infinite loop
  camera.manual();
  camera.storages();
  camera.capture_image();

  loop {
    let event = camera.wait_event(Duration::from_secs(10)).unwrap();

    println!("Event: {:?}", event);

    if event == CameraEvent::Timeout {
      break;
    }
  }

  Ok(())
}
