//! List of cameras and ports

use crate::{helper::chars_to_string, try_gp_internal, Result};
use std::{ops::Range, os::raw::c_int};

pub(crate) struct CameraList {
  pub(crate) inner: *mut libgphoto2_sys::CameraList,
}

impl Drop for CameraList {
  fn drop(&mut self) {
    try_gp_internal!(gp_list_unref(self.inner).unwrap());
  }
}

impl CameraList {
  pub(crate) fn new() -> Result<Self> {
    try_gp_internal!(gp_list_new(&out list)?);

    Ok(Self { inner: list })
  }

  fn range(&self) -> Range<c_int> {
    0..unsafe { libgphoto2_sys::gp_list_count(self.inner) }
  }

  fn get_name_at_unchecked(&self, i: c_int) -> String {
    try_gp_internal!(gp_list_get_name(self.inner, i, &out name).unwrap());
    chars_to_string(name)
  }

  fn get_value_at_unchecked(&self, i: c_int) -> String {
    try_gp_internal!(gp_list_get_value(self.inner, i, &out value).unwrap());
    chars_to_string(value)
  }
}

macro_rules! camera_list_iter {
  ($(# $attr:tt)* |$self:ident: $ty:ident, $i:ident| -> $item_ty:ty $get_item:block) => {
    $(# $attr)*
    pub struct $ty {
      list: CameraList,
      range: Range<c_int>,
    }

    impl $ty {
      pub(crate) fn new(list: CameraList) -> Self {
        Self { range: list.range(), list }
      }
    }

    impl Iterator for $ty {
      type Item = $item_ty;

      fn next(&mut $self) -> Option<Self::Item> {
        $self.range.next().map(|$i| $get_item)
      }

      fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
      }
    }

    impl ExactSizeIterator for $ty {
      fn len(&self) -> usize {
        self.range.len()
      }
    }
  };
}

/// Descriptor representing model+port pair of the connected camera.
#[derive(Debug)]
pub struct CameraDescriptor {
  /// Camera model.
  pub model: String,
  /// Port the camera is connected to.
  pub port: String,
}

camera_list_iter!(
  /// Iterator over camera names and ports.
  |self: CameraListIter, i| -> CameraDescriptor {
    CameraDescriptor {
      model: self.list.get_name_at_unchecked(i),
      port: self.list.get_value_at_unchecked(i),
    }
  }
);

camera_list_iter!(
  /// Iterator over filenames.
  |self: FileListIter, i| -> String { self.list.get_name_at_unchecked(i) }
);
