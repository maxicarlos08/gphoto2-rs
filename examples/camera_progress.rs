use gphoto2::{Context, Result};
use std::collections::HashMap;

enum ProgressMessage {
  Start(f32, String, u64),
  Update(u32, f32),
  Stop(u32),
}
// TODO: Finish this example
fn main() -> Result<()> {
  let mut context = Context::new()?;

  let (sender, receiver) = std::sync::mpsc::channel::<ProgressMessage>();

  context.set_progress_functions(
    Box::new(move |target, message| {
      println!("target: {target}, message: {message}");
      todo!();
    }),
    Box::new(|id, current| {
      println!("id: {id}, current: {current}");
    }),
    Box::new(|id| println!("{id} has ended")),
  );

  Ok(())
}
