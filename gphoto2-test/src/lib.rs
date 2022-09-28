use std::os::raw::{c_char, c_int};
use std::path::Path;

macro_rules! c_str_concat {
  ($($s:expr),*) => {
    std::ffi::CStr::from_bytes_with_nul(concat!($($s,)* "\0").as_bytes())
      .unwrap()
      .as_ptr()
      .cast::<c_char>()
  };
}

/// Set environment variables for libgphoto2.
///
/// Currently this provides the location for the virtual camera's filesystem.
pub fn set_env() {
  // Fun fact: `std::env::set_var` is not guaranteed to end up in the libc
  // environment table (it doesn't on Windows, where `SetEnvironmentVariableW`
  // is used by the implementation), and `setenv` doesn't exist on Windows
  // either, so we have to use `putenv` for our changes to be noticed by
  // libgphoto2 on all platforms.
  extern "C" {
    fn putenv(s: *const c_char) -> c_int;
  }

  // Be careful: in some implementations `putenv` expects the input string
  // to live as long as it's in the environment table.
  //
  // In this case we're fine because all strings are constant (concatenated
  // at compile-time).
  unsafe {
    putenv(c_str_concat!("VCAMERADIR=", env!("OUT_DIR"), "/vcamera"));
  }
}

pub fn libgphoto2_dir() -> &'static Path {
  Path::new(concat!(env!("OUT_DIR"), "/install"))
}

pub fn vcamera_dir() -> &'static Path {
  Path::new(concat!(env!("OUT_DIR"), "/vcamera"))
}

pub const SAMPLE_IMAGE: &[u8] = include_bytes!("../blank.jpg");
