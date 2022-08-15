//! # Gphoto2-rs
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

// TODO: Camera abilities
// TODO: Ports
// TODO: Settings (get, set)
// TODO: Use PhantomData for safety

#![deny(unused_must_use)]
#![warn(missing_docs)]

pub mod abilities;
pub mod camera;
pub mod context;
pub mod error;
pub mod file;
pub(crate) mod helper;
pub mod list;
pub mod port;

pub use crate::context::Context;

/// Raw bindings to libgphoto2.
///
/// Use this at your own risk
pub use libgphoto2_sys;

use error::{Error, Result};
