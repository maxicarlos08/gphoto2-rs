//! Widgets are used to configure the camera

use crate::{helper::uninit, try_gp_internal, Result};
use std::{
  borrow::Cow,
  ffi,
  marker::PhantomData,
  os::raw::{c_char, c_int},
};

/// Type of a widget
pub enum WidgetType {
  /// Top-level configuration object
  Window,
  /// Configuration section
  Section,
  /// Text configuration
  Text,
  /// Range configuration
  Range {
    /// Minimum value
    min: f64,
    /// Maximum value
    max: f64,
    /// Step
    increment: f64,
  },
  /// Boolean
  Toggle,
  ///
  Radio {
    choices: Vec<String>,
    menu: bool,
  },
  Button,
  Date(String),
}

/// A configuration widget
pub struct Widget<'a> {
  pub(crate) inner: *mut libgphoto2_sys::CameraWidget,
  _phantom: PhantomData<&'a ffi::c_void>,
}

impl Drop for Widget<'_> {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_widget_unref(self.inner);
    }
  }
}

impl<'a> Widget<'a> {
  pub(crate) fn new(widget: *mut libgphoto2_sys::CameraWidget) -> Self {
    Self { inner: widget, _phantom: PhantomData }
  }

  /// Counts the children of the widget
  pub fn children_count(&self) -> Result<usize> {
    try_gp_internal!(libgphoto2_sys::gp_widget_count_children(self.inner))
      .map(|count| count as usize)
  }

  /// Gets a child by its index
  pub fn get_child(&self, index: usize) -> Result<Widget<'a>> {
    let mut child = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_child(self.inner, index as c_int, &mut child))?;

    Ok(Self::new(child))
  }

  /// Get a child by its id
  pub fn get_child_by_id(&self, id: usize) -> Result<Widget<'a>> {
    let mut child = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_child_by_id(
      self.inner,
      id as c_int,
      &mut child
    ))?;

    Ok(Self::new(child))
  }

  /// Get a child by its label
  pub fn get_child_by_label(&self, label: &str) -> Result<Widget<'a>> {
    let mut child = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_child_by_label(
      self.inner,
      label.as_ptr() as *const c_char,
      &mut child
    ))?;

    Ok(Self::new(child))
  }

  /// Get a child by its name
  pub fn get_child_by_name(&self, name: &str) -> Result<Widget<'a>> {
    let mut child = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_child_by_name(
      self.inner,
      name.as_ptr() as *const c_char,
      &mut child
    ))?;

    Ok(Self::new(child))
  }
}
