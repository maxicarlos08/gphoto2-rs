#![doc = include_str!("../README.md")]
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

/// Trait to get the underlying libgphoto2 pointer of a wrapper
pub trait InnerPtr<T> {
  /// Get a reference to the inner *mut raw pointer
  ///
  /// # Safety
  ///
  /// Interacting with the underlying libgphoto2 pointers can be dangerous, **use this at your own risk**
  unsafe fn inner_mut_ptr(&self) -> &*mut T;
}

/// Trait to get the underlying libgphoto2 value of a wrapper
pub trait Inner<T> {
  /// Get a reference to the inner value
  ///
  /// # Safety
  ///
  /// Interacting with the underlying libgphoto2 values can be dangerous, **use this at your own risk**
  unsafe fn inner(&self) -> &T;
}
