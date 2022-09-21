//! List of cameras and ports

use crate::{
  helper::{as_ref, chars_to_string},
  try_gp_internal, Result,
};
use std::os::raw::c_int;

/// List of string tuples
pub struct CameraList {
  pub(crate) inner: *mut libgphoto2_sys::CameraList,
}

impl Drop for CameraList {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_list_unref(self.inner);
    }
  }
}

as_ref!(CameraList -> libgphoto2_sys::CameraList, *self.inner);

impl CameraList {
  pub(crate) fn new() -> Result<Self> {
    try_gp_internal!(gp_list_new(&out list));

    Ok(Self { inner: list })
  }

  /// Get length of the list.
  pub fn len(&self) -> usize {
    unsafe { libgphoto2_sys::gp_list_count(self.inner) as usize }
  }

  /// Check if the list is empty.
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Get the name and value of the list item at the given index.
  pub fn get_name_value(&self, index: usize) -> Result<(String, String)> {
    try_gp_internal!(gp_list_get_name(self.inner, index as c_int, &out name));
    try_gp_internal!(gp_list_get_value(self.inner, index as c_int, &out value));

    Ok((chars_to_string(name), chars_to_string(value)))
  }

  /// Get a referential iterator.
  pub fn iter(&self) -> CameraListIter<'_> {
    self.into_iter()
  }
}

impl std::fmt::Debug for CameraList {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_list().entries(self.iter()).finish()
  }
}

impl<'a> IntoIterator for &'a CameraList {
  type Item = (String, String);
  type IntoIter = CameraListIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    CameraListIter::new(self)
  }
}

/// Iterator for [`CameraList`]
pub struct CameraListIter<'a> {
  list: &'a CameraList,
  range: std::ops::Range<usize>,
}

impl<'a> CameraListIter<'a> {
  fn new(list: &'a CameraList) -> Self {
    Self { range: 0..list.len(), list }
  }
}

impl Iterator for CameraListIter<'_> {
  type Item = (String, String);

  fn next(&mut self) -> Option<Self::Item> {
    self.range.next().map(|i| self.list.get_name_value(i).unwrap())
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.range.size_hint()
  }
}

impl ExactSizeIterator for CameraListIter<'_> {
  fn len(&self) -> usize {
    self.range.len()
  }
}
