use crate::{camera::Camera, error::Error, try_gp_internal, AsPtr, Result};
use std::{borrow::Cow, ffi, fs, mem::MaybeUninit, os::unix::io::AsRawFd, path::Path};

pub struct CameraFilePath {
  file_path: libgphoto2_sys::CameraFilePath,
}

pub struct CameraFile {
  file: *mut libgphoto2_sys::CameraFile,
}

impl Drop for CameraFile {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_file_unref(self.file);
    }
  }
}

impl AsPtr<libgphoto2_sys::CameraFile> for CameraFile {
  unsafe fn as_ptr(&self) -> *const libgphoto2_sys::CameraFile {
    self.file
  }

  unsafe fn as_mut_ptr(&mut self) -> *mut libgphoto2_sys::CameraFile {
    self.file
  }
}

impl From<libgphoto2_sys::CameraFilePath> for CameraFilePath {
  fn from(file_path: libgphoto2_sys::CameraFilePath) -> Self {
    Self { file_path }
  }
}

impl CameraFilePath {
  pub fn folder(&self) -> Cow<str> {
    unsafe {
      String::from_utf8_lossy(ffi::CStr::from_ptr(self.file_path.folder.as_ptr()).to_bytes())
    }
  }

  fn to_camera_file(&self, camera: &Camera, path: Option<&Path>) -> Result<CameraFile> {
    let mut camera_file = match path {
      Some(dest_path) => CameraFile::new_file(dest_path)?,
      None => CameraFile::new()?,
    };

    try_gp_internal!(libgphoto2_sys::gp_camera_file_get(
      camera.camera,
      self.file_path.folder.as_ptr(),
      self.file_path.name.as_ptr(),
      libgphoto2_sys::CameraFileType::GP_FILE_TYPE_NORMAL,
      camera_file.as_mut_ptr(),
      camera.context
    ))?;

    Ok(camera_file)
  }

  /// Downloads the file to memory
  pub fn get(&self, camera: &Camera) -> Result<CameraFile> {
    self.to_camera_file(camera, None)
  }

  /// Downloads the file to disk
  pub fn download(&self, camera: &Camera, path: &Path) -> Result<CameraFile> {
    self.to_camera_file(camera, Some(path))
  }
}

impl CameraFile {
  pub fn new() -> Result<Self> {
    let mut camera_file_ptr = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_file_new(&mut camera_file_ptr))?;

    Ok(Self { file: camera_file_ptr })
  }

  pub fn new_file(path: &Path) -> Result<Self> {
    if path.is_file() {
      return Err(Error::new(libgphoto2_sys::GP_ERROR_FILE_EXISTS));
    }

    let file = fs::File::create(path)?.as_raw_fd();

    let mut camera_file_ptr = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_file_new_from_fd(&mut camera_file_ptr, file))
      .map(|_| Self { file: camera_file_ptr })
  }

  pub fn get_data(&self) -> Result<Box<[u8]>> {
    let mut size = unsafe { MaybeUninit::zeroed().assume_init() };
    let mut data = unsafe { MaybeUninit::zeroed().assume_init() }; // data from gphoto is returned as i8, but we use it as u8. This might cause errors in future

    try_gp_internal!(libgphoto2_sys::gp_file_get_data_and_size(self.file, &mut data, &mut size))?;

    let data_slice: Box<[u8]> =
      unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) }.into();

    Ok(data_slice)
  }
}
