//! Files stored on camera

#[cfg(feature = "serde")]
use serde::ser::SerializeMap;

use crate::{
  error::Error,
  helper::{as_ref, char_slice_to_cow, chars_to_string, IntoUnixFd},
  task::{BackgroundPtr, Task},
  try_gp_internal, Context, Result,
};
use std::{borrow::Cow, fmt, fs, path::Path};

/// Represents a path of a file on a camera
pub struct CameraFilePath {
  pub(crate) inner: Box<libgphoto2_sys::CameraFilePath>,
}

/// Type of a file
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
pub struct CameraFile {
  pub(crate) inner: BackgroundPtr<libgphoto2_sys::CameraFile>,
  pub(crate) is_from_disk: bool,
}

impl Drop for CameraFile {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_file_unref(*self.inner);
    }
  }
}

impl Clone for CameraFile {
  fn clone(&self) -> Self {
    try_gp_internal!(gp_file_ref(*self.inner).unwrap());

    Self { inner: self.inner, is_from_disk: self.is_from_disk }
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

#[cfg(feature = "serde")]
impl serde::Serialize for CameraFilePath {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut m = serializer.serialize_map(Some(2))?;
    m.serialize_entry("name", &self.name())?;
    m.serialize_entry("folder", &self.folder())?;

    m.end()
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

as_ref!(CameraFile -> libgphoto2_sys::CameraFile, **self.inner);

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

    Ok(Self { inner: BackgroundPtr(camera_file_ptr), is_from_disk: false })
  }

  pub(crate) fn new_file(path: &Path) -> Result<Self> {
    if path.is_file() {
      return Err(Error::new(libgphoto2_sys::GP_ERROR_FILE_EXISTS, None));
    }

    let fd = fs::File::create(path)?.into_unix_fd();

    try_gp_internal!(gp_file_new_from_fd(&out camera_file_ptr, fd)?);
    Ok(Self { inner: BackgroundPtr(camera_file_ptr), is_from_disk: true })
  }

  /// Get the data of the file
  pub fn get_data(&self, context: impl AsRef<Context>) -> Task<Result<Box<[u8]>>> {
    let file = self.clone();

    unsafe {
      Task::new(move || {
        try_gp_internal!(gp_file_get_data_and_size(*file.inner, &out data, &out size)?);

        let data_slice: Box<[u8]> =
          std::slice::from_raw_parts(data.cast::<u8>(), size.try_into()?).into();

        if file.is_from_disk {
          // Casting a *const pointer to *mut is still unstable
          #[allow(clippy::as_conversions)]
          libc::free((data as *mut i8).cast())
        }

        Ok(data_slice)
      })
    }
    .context(context.as_ref().inner)
  }

  /// File name
  pub fn name(&self) -> String {
    try_gp_internal!(gp_file_get_name(*self.inner, &out file_name).unwrap());

    chars_to_string(file_name)
  }

  /// File mime type
  pub fn mime_type(&self) -> String {
    try_gp_internal!(gp_file_get_mime_type(*self.inner, &out mime_type).unwrap());

    chars_to_string(mime_type)
  }

  /// File modification time
  pub fn mtime(&self) -> libc::time_t {
    try_gp_internal!(gp_file_get_mtime(*self.inner, &out mtime).unwrap());

    mtime
  }

  /// File size
  pub fn size(&self, context: &Context) -> Task<Result<u64>> {
    let file = self.clone().inner;

    unsafe {
      Task::new(move || {
        try_gp_internal!(gp_file_get_data_and_size(*file, std::ptr::null_mut(), &out size)?);

        #[allow(clippy::useless_conversion)] // c_ulong depends on the platform
        Ok(size.into())
      })
    }
    .context(context.inner)
  }
}

impl fmt::Debug for CameraFile {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CameraFile")
      .field("name", &self.name())
      .field("mime_type", &self.mime_type())
      .field("mtime", &self.mtime())
      .finish()
  }
}
