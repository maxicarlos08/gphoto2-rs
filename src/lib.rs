// TODO: Documentation

// TODO: Camera abilities
// TODO: Ports
// TODO: Settings (get, set)
// TODO: Use PhantomData for safety

#![deny(unused_must_use)]

pub mod abilities;
pub mod camera;
pub mod context;
pub mod error;
pub mod file;
pub(crate) mod helper;
pub mod list;
pub mod port;

/// Raw bindings to libgphoto2.
/// 
/// Use this at your own risk
pub use libgphoto2_sys;

pub use crate::context::Context;

use error::{Error, Result};

pub enum OwnedOrRef<'a, T> {
  Owned(T),
  Ref(&'a T),
}

trait AsPtr<T> {
  unsafe fn as_ptr(&self) -> *const T;

  unsafe fn as_mut_ptr(&mut self) -> *mut T;
}
