//! GPhoto cannot implement all features of all cameras,
//! but you can send custom PTP opcodes to the camera.
//!
//! This example starts and ends live view mode on the camera for 10 s,
//! this example only works on Nikon cameras.

use gphoto2::{widget::WidgetValue, Context, Result};
use std::{thread, time::Duration};

fn main() -> Result<()> {
  let context = Context::new()?;
  let camera = context.autodetect_camera()?;

  let mut opcode = camera.config_key("opcode")?;

  println!("Starting live view");
  opcode.set_value(WidgetValue::Text("0x9201".into()))?;
  camera.set_config(&opcode)?;

  thread::sleep(Duration::from_secs(10));

  println!("Ending live view");
  opcode.set_value(WidgetValue::Text("0x9202".into()))?;
  camera.set_config(&opcode)?;

  Ok(())
}
