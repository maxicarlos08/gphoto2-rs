//! # GPhoto2-rs
//!
//! High lever abstractions for libgphoto2
//!
//! ## Usage
//!
//! ```
//! use gphoto2::Context;
//!
//! let context = Context::new().expect("Failed to create context"); // Create library context
//! let camera = context.autodetect_camera().expect("Failed to autodetect camera");
//!
//! let file_path = camera.capture_image();
//! file_path.download(&camera, "image.jpg");
//! ```

// TODO: FileSystem

#![deny(unused_must_use)]
#![deny(missing_docs)] // Force documentation on all public API's

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

pub use crate::{
  context::Context,
  error::{Error, Result},
};

/// Raw bindings to libgphoto2.
///
/// Use this at your own risk
pub use libgphoto2_sys;
