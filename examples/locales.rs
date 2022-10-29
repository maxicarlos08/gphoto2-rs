//! Run this example with your system locale set to something non UTF-8.

use gphoto2::{widget::TextWidget, Context, Result};

const CONFIG_KEY: &str = "d406";
const NON_UNICODE_VALUE: &str = "貓"; // "cat" in chinese (because they are awesome)

fn main() -> Result<()> {
  let camera = Context::new()?.autodetect_camera()?;

  let text_widget: TextWidget = camera.config_key(CONFIG_KEY)?;
  text_widget.set_value(NON_UNICODE_VALUE)?;

  camera.set_config(&text_widget)?;

  let text_widget: TextWidget = camera.config_key(CONFIG_KEY)?;

  assert_eq!(text_widget.name(), NON_UNICODE_VALUE);
  Ok(())
}
