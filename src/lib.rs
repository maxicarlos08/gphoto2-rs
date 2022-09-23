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
