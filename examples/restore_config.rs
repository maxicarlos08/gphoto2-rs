//! Recursively list all configuration
//! Warning: Output might be very large

use gphoto2::{widget::RadioWidget, Context, Result};
use log::{error, info, LevelFilter};

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();

  let camera = Context::new()?.autodetect_camera().wait()?;

  info!("saving user configuration...");
  let conf = camera.config().wait()?;

  let shutter_speed = camera.config_key::<RadioWidget>("shutterspeed").wait()?;
  let initial_choice = shutter_speed.choice();
  info!("initial shutter_speed: {initial_choice}");

  if initial_choice == "bulb" {
    shutter_speed.set_choice("5")?;
  } else {
    shutter_speed.set_choice("bulb")?;
  }
  camera.set_config(&shutter_speed).wait()?;

  let shutter_speed = camera.config_key::<RadioWidget>("shutterspeed").wait()?;
  info!("modified shutter_speed: {}", shutter_speed.choice());

  info!("restoring initial config...");
  camera.set_all_config(&conf).wait()?;

  let shutter_speed = camera.config_key::<RadioWidget>("shutterspeed").wait()?;
  let final_choice = shutter_speed.choice();
  info!("final shutter_speed: {final_choice}");

  if initial_choice == final_choice {
    info!("config was unchanged");
    Ok(())
  } else {
    error!("config has changed: restoration has failed");
    Err(gphoto2::Error::new(-1, Some("Unexpected behavior".to_string())))
  }
}
