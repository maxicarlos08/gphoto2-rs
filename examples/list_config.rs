//! Recursively list all configuration
//! Warning: Output might be very large

use gphoto2::{widget::Widget, Context, Result};

fn display_widget_recursive(widget: &Widget, prefix: &str) -> Result<()> {
  let name = widget.name()?;
  let id = widget.id()?;
  let new_prefix = format!("{prefix}/{name}");

  println!("{} ({})", new_prefix, id);
  println!("LABEL: {}", widget.label()?);
  println!("READONLY: {}", widget.readonly()?);
  println!("TYPE: {:#?}", widget.widget_type()?);

  if let Ok((Some(value), _)) = widget.value() {
    println!("VALUE: {:?}", value)
  }

  print!("\n");

  for child in widget.children_iter()? {
    display_widget_recursive(&child, &new_prefix)?;
  }

  Ok(())
}

fn main() -> Result<()> {
  let context = Context::new()?;
  let camera = context.autodetect_camera()?;

  display_widget_recursive(&camera.config()?, "")?;

  Ok(())
}
