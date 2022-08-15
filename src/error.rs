use std::{error, ffi, fmt, os::raw::c_int};

pub type Result<T> = std::result::Result<T, Error>;

pub const GP_OK: i32 = libgphoto2_sys::GP_OK as i32;

#[derive(Debug)]
pub enum ErrorKind {
  // GPhoto Errors
  Other,
  BadParameters,
  CameraBusy,
  CameraError,
  CorruptedData,
  DirectoryExists,
  DirectoryNotFound,
  FileExists,
  FileNotFound,
  FixedLimitExceeded,
  ModelNotFound,
  NotSupported,
  NoMemory,
  NoSpace,
  Io,
  OsFailure,
  PathNotAbsolute,
  Timeout,
  UnknownPort,
}

#[derive(Debug)]
pub struct Error {
  error: c_int,
}

impl Error {
  pub fn new(error: c_int) -> Self {
    Self { error }
  }

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
  fn from(_: std::io::Error) -> Self {
    // TODO: IO errors should have more detail

    Self { error: libgphoto2_sys::GP_ERROR_IO }
  }
}

impl From<std::ffi::NulError> for Error {
  fn from(_: std::ffi::NulError) -> Self {
    Self { error: libgphoto2_sys::GP_ERROR }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(unsafe {
      ffi::CStr::from_ptr(libgphoto2_sys::gp_result_as_string(self.error))
        .to_str()
        .unwrap_or("Invalid error message")
    })
  }
}

impl error::Error for Error {}

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
