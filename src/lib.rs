#![doc = include_str!("../README.md")]
#![deny(unused_must_use)]
#![deny(missing_docs)] // Force documentation on all public API's
#![deny(clippy::as_conversions)]

pub mod abilities;
pub mod camera;
pub mod context;
pub mod error;
pub mod file;
pub mod filesys;
pub(crate) mod helper;
pub mod list;
pub mod port;
pub mod task;
pub(crate) mod thread;
pub mod widget;

use std::ffi::CStr;

use self::error::try_gp_internal;

#[doc(inline)]
pub use crate::{
  camera::Camera,
  context::Context,
  error::{Error, Result},
};

/// Raw bindings to libgphoto2.
///
/// Use this at your own risk
pub use libgphoto2_sys;

#[cfg(all(test, not(feature = "test")))]
compile_error!("The test feature must be enabled to run the tests");

/// Get the short version of the libgphoto2 library used
pub fn library_version() -> Option<&'static str> {
  unsafe {
    CStr::from_ptr(*libgphoto2_sys::gp_library_version(
      libgphoto2_sys::GPVersionVerbosity::GP_VERSION_SHORT,
    ))
    .to_str()
    .ok()
  }
}

#[cfg(all(test, feature = "test"))]
fn sample_context() -> Context {
  use std::sync::Once;

  static INIT: Once = Once::new();
  INIT.call_once(|| {
    // Enable logging.
    env_logger::builder()
      // As much logging as possible.
      .filter_module("gphoto2", log::LevelFilter::max())
      // But hide logs if tests are successful.
      .is_test(true)
      .init();

    // Tell libgphoto2 to look for drivers in a custom built directory.
    libgphoto2_sys::test_utils::set_env();
  });

  Context::new().unwrap()
}

#[cfg(all(test, feature = "test"))]
#[test]
fn test_version() {
  insta::assert_snapshot!(library_version().unwrap());
}
