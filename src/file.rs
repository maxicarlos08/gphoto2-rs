//! Files stored on camera

use crate::{
  error::Error,
  helper::{as_ref, char_slice_to_cow, chars_to_string, to_c_string, IntoUnixFd},
  try_gp_internal, Result,
};
use std::{borrow::Cow, ffi, fmt, fs, path::Path};

/// Represents a path of a file on a camera
pub struct CameraFilePath {
  pub(crate) inner: Box<libgphoto2_sys::CameraFilePath>,
}

/// Type of a file
pub enum FileType {
  /// Preview of an image
  Preview,
  /// Normal fil
  Normal,
  /// Raw data before postprocessing of driver, RAW image files are usually [`FileType::Normal`]
  Raw,
  /// Audio contained in file
  Audio,
  /// Embedded EXIF data of an image
  Exif,
  /// Metadata of a file
  Metadata,
}

/// File on a camera
///
/// To download the file use [`CameraFS`](crate::filesys::CameraFS)
/// ```
pub struct CameraFile {
  pub(crate) inner: *mut libgphoto2_sys::CameraFile,
  pub(crate) is_from_disk: bool,
}

impl Drop for CameraFile {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_file_unref(self.inner);
    }
  }
}

impl From<libgphoto2_sys::CameraFileType> for FileType {
  fn from(file_type: libgphoto2_sys::CameraFileType) -> Self {
    use libgphoto2_sys::CameraFileType as GPFileType;

    match file_type {
      GPFileType::GP_FILE_TYPE_PREVIEW => Self::Preview,
      GPFileType::GP_FILE_TYPE_NORMAL => Self::Normal,
      GPFileType::GP_FILE_TYPE_RAW => Self::Raw,
      GPFileType::GP_FILE_TYPE_AUDIO => Self::Audio,
      GPFileType::GP_FILE_TYPE_EXIF => Self::Exif,
      GPFileType::GP_FILE_TYPE_METADATA => Self::Metadata,
    }
  }
}

#[allow(clippy::from_over_into)]
impl Into<libgphoto2_sys::CameraFileType> for FileType {
  fn into(self) -> libgphoto2_sys::CameraFileType {
    use libgphoto2_sys::CameraFileType as GPFileType;

    match self {
      Self::Preview => GPFileType::GP_FILE_TYPE_PREVIEW,
      Self::Normal => GPFileType::GP_FILE_TYPE_NORMAL,
      Self::Raw => GPFileType::GP_FILE_TYPE_RAW,
      Self::Audio => GPFileType::GP_FILE_TYPE_AUDIO,
      Self::Exif => GPFileType::GP_FILE_TYPE_EXIF,
      Self::Metadata => GPFileType::GP_FILE_TYPE_METADATA,
    }
  }
}

impl fmt::Debug for CameraFilePath {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CameraFilePath")
      .field("folder", &self.folder())
      .field("name", &self.name())
      .finish()
  }
}

as_ref!(CameraFile -> libgphoto2_sys::CameraFile, *self.inner);

as_ref!(CameraFilePath -> libgphoto2_sys::CameraFilePath, self.inner);

impl CameraFilePath {
  /// Get the name of the file's folder
  pub fn folder(&self) -> Cow<str> {
    char_slice_to_cow(&self.inner.folder)
  }

  /// Get the basename of the file (without the folder)
  pub fn name(&self) -> Cow<str> {
    char_slice_to_cow(&self.inner.name)
  }
}

impl CameraFile {
  pub(crate) fn new() -> Result<Self> {
    try_gp_internal!(gp_file_new(&out camera_file_ptr)?);

    Ok(Self { inner: camera_file_ptr, is_from_disk: false })
  }

  pub(crate) fn new_file(path: &Path) -> Result<Self> {
    if path.is_file() {
      return Err(Error::new(libgphoto2_sys::GP_ERROR_FILE_EXISTS, None));
    }

    let fd = fs::File::create(path)?.into_unix_fd();

    try_gp_internal!(gp_file_new_from_fd(&out camera_file_ptr, fd)?);
    Ok(Self { inner: camera_file_ptr, is_from_disk: true })
  }

  /// Creates a new camera file from disk
  pub fn new_from_disk(path: &Path) -> Result<Self> {
    try_gp_internal!(gp_file_new_from_fd(&out camera_file_ptr, -1)?);
    try_gp_internal!(gp_file_open(
      camera_file_ptr,
      to_c_string!(path.to_str().ok_or("File path invalid")?)
    )?);

    Ok(Self { inner: camera_file_ptr, is_from_disk: true })
  }

  /// Get the data of the file
  pub fn get_data(&self) -> Result<Box<[u8]>> {
    try_gp_internal!(gp_file_get_data_and_size(self.inner, &out data, &out size)?);

    let data_slice: Box<[u8]> =
      unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size.try_into()?) }.into();

    if self.is_from_disk {
      unsafe {
        // Casting a *const pointer to *mut is still unstable
        #[allow(clippy::as_conversions)]
        libc::free((data as *mut i8).cast())
      }
    }

    Ok(data_slice)
  }

  /// File name
  pub fn name(&self) -> String {
    try_gp_internal!(gp_file_get_name(self.inner, &out file_name).unwrap());

    chars_to_string(file_name)
  }

  /// File mime type
  pub fn mime(&self) -> String {
    try_gp_internal!(gp_file_get_mime_type(self.inner, &out mime).unwrap());

    chars_to_string(mime)
  }
}
