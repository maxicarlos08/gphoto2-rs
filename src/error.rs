//! Error handling

use crate::helper::chars_to_cow;
use std::{error, fmt, os::raw::c_int};

/// Result type used in this library
pub type Result<T> = std::result::Result<T, Error>;

/// i32 version of [`libgphoto2_sys::GP_OK`]
pub const GP_OK: c_int = libgphoto2_sys::GP_OK as c_int;

/// Error type
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ErrorKind {
  /// GP_ERROR or something else
  Other,
  /// Bad parameters were used
  BadParameters,
  /// The camera is bsy
  CameraBusy,
  /// The camera returned an error
  CameraError,
  /// Corrupted data
  CorruptedData,
  /// The directory already exists
  DirectoryExists,
  /// The directory was not found
  DirectoryNotFound,
  /// The file already exists
  FileExists,
  /// The file was not found
  FileNotFound,
  /// Limit exceeded
  FixedLimitExceeded,
  /// Camera model not found
  ModelNotFound,
  /// Action not supported
  NotSupported,
  /// Memory error
  NoMemory,
  /// Not enough space
  NoSpace,
  /// Io error
  Io,
  /// OS error
  OsFailure,
  /// Path is not absolute
  PathNotAbsolute,
  /// Timeout
  Timeout,
  /// Port is not known
  UnknownPort,
}

/// General error
#[derive(PartialEq)]
pub struct Error {
  error: c_int,
  info: Option<String>,
}

impl Error {
  /// Creates a new error from a gphoto internal error
  pub fn new(error: c_int) -> Self {
    Self { error, info: None }
  }

  /// Map the gphoto type to an [`ErrorKind`]
  pub fn kind(&self) -> ErrorKind {
    match self.error {
      libgphoto2_sys::GP_ERROR_BAD_PARAMETERS => ErrorKind::BadParameters,
      libgphoto2_sys::GP_ERROR_CAMERA_BUSY => ErrorKind::CameraBusy,
      libgphoto2_sys::GP_ERROR_CAMERA_ERROR => ErrorKind::CameraError,
      libgphoto2_sys::GP_ERROR_CORRUPTED_DATA => ErrorKind::CorruptedData,
      libgphoto2_sys::GP_ERROR_DIRECTORY_EXISTS => ErrorKind::DirectoryExists,
      libgphoto2_sys::GP_ERROR_DIRECTORY_NOT_FOUND => ErrorKind::DirectoryNotFound,
      libgphoto2_sys::GP_ERROR_FILE_EXISTS => ErrorKind::FileExists,
      libgphoto2_sys::GP_ERROR_FILE_NOT_FOUND => ErrorKind::FileNotFound,
      libgphoto2_sys::GP_ERROR_FIXED_LIMIT_EXCEEDED => ErrorKind::FixedLimitExceeded,
      libgphoto2_sys::GP_ERROR_MODEL_NOT_FOUND => ErrorKind::ModelNotFound,
      libgphoto2_sys::GP_ERROR_NOT_SUPPORTED => ErrorKind::NotSupported,
      libgphoto2_sys::GP_ERROR_NO_MEMORY => ErrorKind::NoMemory,
      libgphoto2_sys::GP_ERROR_NO_SPACE => ErrorKind::NoSpace,
      libgphoto2_sys::GP_ERROR_IO => ErrorKind::Io,
      libgphoto2_sys::GP_ERROR_OS_FAILURE => ErrorKind::OsFailure,
      libgphoto2_sys::GP_ERROR_PATH_NOT_ABSOLUTE => ErrorKind::PathNotAbsolute,
      libgphoto2_sys::GP_ERROR_TIMEOUT => ErrorKind::Timeout,
      libgphoto2_sys::GP_ERROR_UNKNOWN_PORT => ErrorKind::UnknownPort,

      libgphoto2_sys::GP_ERROR | _ => ErrorKind::Other,
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Self { error: libgphoto2_sys::GP_ERROR_IO, info: Some(err.to_string()) }
  }
}

impl From<std::ffi::NulError> for Error {
  fn from(_: std::ffi::NulError) -> Self {
    Self { error: libgphoto2_sys::GP_ERROR, info: Some("FFI: NulError".to_string()) }
  }
}

impl From<&str> for Error {
  fn from(message: &str) -> Self {
    Self { error: libgphoto2_sys::GP_ERROR, info: Some(message.into()) }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(unsafe { &chars_to_cow(libgphoto2_sys::gp_result_as_string(self.error)) })?;

    if let Some(error_info) = &self.info {
      f.write_fmt(format_args!(" [{}]", error_info))?;
    }

    Ok(())
  }
}

impl fmt::Debug for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    <Self as fmt::Display>::fmt(self, f)
  }
}

impl error::Error for Error {}

/// Check the result of an internal libgphoto2 function.
///
/// If the return type is less than 0, an error is returned,
/// otherwise the result of the function
#[macro_export]
macro_rules! try_gp_internal {
  ($x:expr) => {{
    let v = unsafe { $x };

    if v >= 0 {
      Ok(v)
    } else {
      Err($crate::Error::new(v))
    }
  }};
}
