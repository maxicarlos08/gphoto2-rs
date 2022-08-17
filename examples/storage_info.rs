use gphoto2::{Context, Result};

fn main() -> Result<()> {
  let context = Context::new()?;
  let camera = context.autodetect_camera()?;

  let storages = camera.storages()?;

  for (i, storage) in storages.iter().enumerate() {
    println!("==== Storage #{} ====", i);
    println!("Label: {:?}", storage.label());
    println!("Internal path: {:?}", storage.base_directory());
    println!("Description: {:?}", storage.description());
    println!("Storage type: {:?}", storage.storage_type());
    println!("Filesystem type: {:?}", storage.filesystem_type());
    println!("Capacity (KB): {:?}", storage.capacity());
    println!("Free (KB): {:?}", storage.free());
    println!("Access permissions: {:?}", storage.access_type());
  }

  Ok(())
}