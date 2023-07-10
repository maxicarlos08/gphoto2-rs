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

#[cfg(all(test, feature = "test"))]
fn sample_context() -> Context {
  use std::sync::Once;
  use tracing_subscriber::{fmt, prelude::*, EnvFilter};

  static INIT: Once = Once::new();
  INIT.call_once(|| {
    tracing_subscriber::registry()
      .with(fmt::layer())
      .with(EnvFilter::from_default_env().add_directive("gphoto2=trace".parse().unwrap()))
      .init();

    // Tell libgphoto2 to look for drivers in a custom built directory.
    libgphoto2_sys::test_utils::set_env();
  });

  Context::new().unwrap()
}
