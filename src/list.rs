//! List of cameras and ports

use crate::{try_gp_internal, Result, helper::{as_ref, chars_to_string}};

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

  /// Converts the internal gphoto list to a rust vec
  pub fn to_vec(&self) -> Result<Vec<(String, String)>> {
    let length = unsafe { libgphoto2_sys::gp_list_count(self.inner) };

    let mut res = Vec::with_capacity(length as usize);

    for list_index in 0..length {
      try_gp_internal!(gp_list_get_name(self.inner, list_index, &out name));
      try_gp_internal!(gp_list_get_value(self.inner, list_index, &out value));

      res.push((chars_to_string(name), chars_to_string(value)));
    }

    Ok(res)
  }
}
