#![doc = include_str!("../README.md")]

#![deny(unused_must_use)]
#![deny(missing_docs)] // Force documentation on all public API's

pub mod abilities;
pub(crate) mod camera;
pub mod context;
pub mod error;
pub mod file;
pub mod filesys;
pub(crate) mod helper;
pub mod list;
pub mod port;
pub mod widget;

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
