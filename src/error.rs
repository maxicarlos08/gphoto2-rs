//! Error handling

use std::{error, fmt, os::raw::c_int};

use crate::helper::chars_to_string;

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
#[derive(PartialEq, Eq)]
pub struct Error {
  error: c_int,
  info: Option<String>,
}

impl Error {
  /// Creates a new error from a gphoto internal error
  pub fn new(error: c_int, info: Option<String>) -> Self {
    Self { error, info }
  }

  /// Checks the status code and creates a new error if non-zero.
  pub(crate) fn check(status: c_int) -> Result<c_int> {
    if status < 0 {
      Err(Self::new(status, None))
    } else {
      Ok(status)
    }
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

      libgphoto2_sys::GP_ERROR => ErrorKind::Other,
      _ => ErrorKind::Other,
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

impl From<std::convert::Infallible> for Error {
  fn from(err: std::convert::Infallible) -> Self {
    match err {}
  }
}

impl From<String> for Error {
  fn from(message: String) -> Self {
    Self { error: libgphoto2_sys::GP_ERROR, info: Some(message) }
  }
}

impl From<&str> for Error {
  fn from(message: &str) -> Self {
    message.to_owned().into()
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(unsafe { &chars_to_string(libgphoto2_sys::gp_result_as_string(self.error)) })?;

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
  (@ $unwrap:tt $status:tt [ $($out:ident)* ] $func:ident ( $($args:tt)* ) &out $new_out:ident $($rest:tt)*) => {
    try_gp_internal!(@ $unwrap $status [ $($out)* $new_out ] $func ( $($args)* $new_out.as_mut_ptr() ) $($rest)*)
  };

  (@ $unwrap:tt $status:tt $out:tt $func:ident ( $($args:tt)* ) $new_arg_token:tt $($rest:tt)*) => {
    try_gp_internal!(@ $unwrap $status $out $func ( $($args)* $new_arg_token ) $($rest)*)
  };

  (@ ($($unwrap:tt)*) $status:tt [ $($out:ident)* ] $func:ident $args:tt) => {
    #[allow(unused_unsafe)]
    let ($status, $($out),*) = unsafe {
      $(let mut $out = std::mem::MaybeUninit::uninit();)*

      let status = match $crate::Error::check(libgphoto2_sys::$func $args) {
        Ok(status) => status,
        Err(err) => {
          return Err(err) $($unwrap)*;
        },
      };

      (status, $($out.assume_init()),*)
    };
  };

  (let $status:tt = $func:ident ( $($args:tt)* ) $($unwrap:tt)*) => {
    try_gp_internal!(@ ($($unwrap)*) $status [] $func () $($args)*)
  };

  ($func:ident ( $($args:tt)* ) $($unwrap:tt)*) => {
    try_gp_internal!(let _ = $func ( $($args)* ) $($unwrap)*)
  };
}
