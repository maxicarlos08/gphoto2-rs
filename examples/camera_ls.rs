use gphoto2::{filesys::CameraFS, Context, Result};
use std::collections::HashMap;

#[derive(Debug)]
#[allow(dead_code)]
struct FolderContent {
  folders: HashMap<String, FolderContent>,
  files: Vec<String>,
}

fn list_folder_recursive(fs: &CameraFS, folder_name: &str) -> Result<FolderContent> {
  let folders_iter = fs.list_folders(folder_name).wait()?;
  let mut folders = HashMap::with_capacity(folders_iter.len());

  for folder in folders_iter {
    let folder_full_name =
      format!("{}/{folder}", if folder_name == "/" { "" } else { folder_name });
    folders.insert(folder, list_folder_recursive(fs, &folder_full_name)?);
  }

  let files = fs.list_files(folder_name).wait()?.collect();

  Ok(FolderContent { files, folders })
}

fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let camera = Context::new()?.autodetect_camera().wait()?;
  let fs = camera.fs();

  let folders = list_folder_recursive(&fs, "/")?;

  println!("{:#?}", folders);
  Ok(())
}
