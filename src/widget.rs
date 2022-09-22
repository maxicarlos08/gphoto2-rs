//! Camera configuration
//!
//! ## Configuring a camera
//! ```no_run
//! use gphoto2::{Context, widget::RadioWidget, Result};
//!
//! # fn main() -> Result<()> {
//! let context = Context::new()?;
//! let camera = context.autodetect_camera()?;
//!
//! let mut config = camera.config_key::<RadioWidget>("iso")?;
//! config.set_choice("100")?; // Set the iso to 100
//! camera.set_config(&config); // Apply setting to camera
//! # Ok(())
//! # }
//! ```

use crate::{
  helper::{as_ref, chars_to_string, to_c_string, FmtResult},
  try_gp_internal, Camera, Error, Result,
};
use std::{
  ffi, fmt,
  ops::{Range, RangeInclusive},
  os::raw::{c_char, c_int, c_void},
};

/// Iterator over the children of a widget
pub struct WidgetIterator<'a> {
  parent_widget: &'a GroupWidget,
  range: Range<usize>,
}

impl<'a> Iterator for WidgetIterator<'a> {
  type Item = Widget;

  fn next(&mut self) -> Option<Self::Item> {
    self.range.next().map(|i| self.parent_widget.get_child(i).unwrap())
  }
}

/// Base widget type providing general information about the widget.
///
/// Normally you shouldn't use this type directly but should acccess its
/// properties via [`Widget`] or specific typed widgets instead.
pub struct WidgetBase {
  pub(crate) inner: *mut libgphoto2_sys::CameraWidget,
}

impl Clone for WidgetBase {
  fn clone(&self) -> Self {
    unsafe {
      libgphoto2_sys::gp_widget_ref(self.inner);
    }

    Self { inner: self.inner }
  }
}

impl Drop for WidgetBase {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_widget_unref(self.inner);
    }
  }
}

impl WidgetBase {
  fn as_ptr(&self) -> *mut libgphoto2_sys::CameraWidget {
    self.inner
  }

  /// Get exact widget type.
  fn ty(&self) -> Result<libgphoto2_sys::CameraWidgetType> {
    try_gp_internal!(gp_widget_get_type(self.inner, &out widget_type));
    Ok(widget_type)
  }

  /// If true, the widget cannot be written
  pub fn readonly(&self) -> Result<bool> {
    try_gp_internal!(gp_widget_get_readonly(self.inner, &out readonly));

    Ok(readonly == 1)
  }

  /// Get the widget label
  pub fn label(&self) -> Result<String> {
    try_gp_internal!(gp_widget_get_label(self.inner, &out label));

    Ok(chars_to_string(label))
  }

  /// Get the widget name
  pub fn name(&self) -> Result<String> {
    try_gp_internal!(gp_widget_get_name(self.inner, &out name));
    Ok(chars_to_string(name))
  }

  /// Get the widget id
  pub fn id(&self) -> Result<i32> {
    try_gp_internal!(gp_widget_get_id(self.inner, &out id));

    Ok(id)
  }

  /// Get information about the widget
  pub fn info(&self) -> Result<String> {
    try_gp_internal!(gp_widget_get_info(self.inner, &out info));

    Ok(chars_to_string(info))
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("id", self.id().fmt_res())
      .field("name", self.name().fmt_res())
      .field("label", self.label().fmt_res())
      .field("readonly", self.readonly().fmt_res());
  }

  unsafe fn raw_value<T>(&self) -> Result<T> {
    try_gp_internal!(gp_widget_get_value(self.inner, &out value as *mut T as *mut c_void));
    Ok(value)
  }

  unsafe fn set_raw_value<T>(&self, value: *const T) -> Result<()> {
    try_gp_internal!(gp_widget_set_value(self.inner, value.cast::<c_void>()));
    Ok(())
  }
}

impl fmt::Debug for WidgetBase {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut f = f.debug_struct("WidgetBase");
    f.field("type", &self.ty());
    self.fmt_fields(&mut f);
    f.finish()
  }
}

as_ref!(WidgetBase -> libgphoto2_sys::CameraWidget, *self.inner);

macro_rules! join_strings {
  ($delim:literal, $first:expr $(, $rest:expr)*) => {
    concat!($first $(, $delim, $rest)*)
  };
}

macro_rules! typed_widgets {
  ($($name:ident, $variant:ident = $($gp_name:ident)|+;)*) => {
    /// Typed widget representation.
    #[derive(Clone)]
    pub enum Widget {
      $(
        #[doc = concat!("Variant representing a [`", stringify!($name), "`].")]
        $variant($name),
      )*
    }

    impl Widget {
      pub(crate) fn new_owned(widget: *mut libgphoto2_sys::CameraWidget) -> Result<Self> {
        let inner = WidgetBase { inner: widget };

        Ok(match inner.ty()? {
          $($(libgphoto2_sys::CameraWidgetType::$gp_name)|+ => Widget::$variant($name { inner }),)*
        })
      }
    }

    $(
      impl From<$name> for Widget {
        fn from(widget: $name) -> Self {
          Widget::$variant(widget)
        }
      }

      impl TryFrom<Widget> for $name {
        type Error = Error;

        fn try_from(widget: Widget) -> Result<Self> {
          match widget {
            Widget::$variant(widget) => Ok(widget),
            _ => Err(Error::from(format!("Expected {} but got {:?}", stringify!($name), widget))),
          }
        }
      }
    )*

    impl Widget {
      /// Try to downcast the widget to the specific type.
      pub fn try_into<T>(self) -> std::result::Result<T, <T as TryFrom<Widget>>::Error>
      where
        T: TryFrom<Widget>,
      {
        TryInto::try_into(self)
      }
    }

    impl fmt::Debug for Widget {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
          $(Self::$variant(widget) => widget.fmt(f),)*
        }
      }
    }

    impl std::ops::Deref for Widget {
      type Target = WidgetBase;

      fn deref(&self) -> &WidgetBase {
        match self {
          $(Self::$variant(widget) => widget),*
        }
      }
    }

    $(
      #[doc = concat!(
        "Widget representing ",
        join_strings!(
          " or "
          $(, concat!(
            "[`", stringify!($gp_name), "`]",
            "(libgphoto2_sys::CameraWidgetType::", stringify!($gp_name), ")"
          ))+
        ),
        "."
      )]
      #[derive(Clone)]
      pub struct $name {
        inner: WidgetBase,
      }

      impl std::fmt::Debug for $name {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          let mut f = f.debug_struct(stringify!($name));
          // Add fields from core WidgetBase.
          self.inner.fmt_fields(&mut f);
          // Add fields widget itself wishes to report.
          $name::fmt_fields(self, &mut f);
          f.finish()
        }
      }

      impl std::ops::Deref for $name {
        type Target = WidgetBase;

        fn deref(&self) -> &WidgetBase {
          &self.inner
        }
      }
    )*
  };
}

typed_widgets!(
  GroupWidget, Group = GP_WIDGET_WINDOW | GP_WIDGET_SECTION;
  TextWidget, Text = GP_WIDGET_TEXT;
  RangeWidget, Range = GP_WIDGET_RANGE;
  ToggleWidget, Toggle = GP_WIDGET_TOGGLE;
  RadioWidget, Radio = GP_WIDGET_MENU | GP_WIDGET_RADIO;
  ButtonWidget, Button = GP_WIDGET_BUTTON;
  DateWidget, Date = GP_WIDGET_DATE;
);

/// Helper that prints `...` when using `{:?}` or the given list when using `{:#?}`.
struct MaybeListFmt<F>(F);

impl<Iter: IntoIterator, F: Fn() -> Result<Iter>> fmt::Debug for MaybeListFmt<F>
where
  Iter::Item: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if f.alternate() {
      match (self.0)() {
        Ok(iter) => f.debug_list().entries(iter).finish(),
        Err(err) => err.fmt(f),
      }
    } else {
      f.write_str("...")
    }
  }
}

impl GroupWidget {
  /// Creates a new [`WidgetIterator`]
  pub fn children_iter(&self) -> Result<WidgetIterator<'_>> {
    Ok(WidgetIterator { parent_widget: self, range: 0..self.children_count()? })
  }

  /// Counts the children of the widget
  pub fn children_count(&self) -> Result<usize> {
    try_gp_internal!(let count = gp_widget_count_children(self.as_ptr()));
    Ok(count as usize)
  }

  /// Gets a child by its index
  pub fn get_child(&self, index: usize) -> Result<Widget> {
    try_gp_internal!(gp_widget_get_child(self.as_ptr(), index as c_int, &out child));

    Widget::new_shared(child)
  }

  /// Get a child by its id
  pub fn get_child_by_id(&self, id: usize) -> Result<Widget> {
    try_gp_internal!(gp_widget_get_child_by_id(self.as_ptr(), id as c_int, &out child));

    Widget::new_shared(child)
  }

  /// Get a child by its label
  pub fn get_child_by_label(&self, label: &str) -> Result<Widget> {
    try_gp_internal!(gp_widget_get_child_by_label(self.as_ptr(), to_c_string!(label), &out child));

    Widget::new_shared(child)
  }

  /// Get a child by its name
  pub fn get_child_by_name(&self, name: &str) -> Result<Widget> {
    try_gp_internal!(gp_widget_get_child_by_name(self.as_ptr(), to_c_string!(name), &out child));

    Widget::new_shared(child)
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("children", &MaybeListFmt(|| self.children_iter()));
  }
}

impl TextWidget {
  /// Get the value of the widget.
  pub fn value(&self) -> Result<String> {
    Ok(chars_to_string(unsafe { self.raw_value::<*const c_char>()? }))
  }

  /// Set the value of the widget.
  pub fn set_value(&self, value: &str) -> Result<()> {
    unsafe { self.set_raw_value::<c_char>(to_c_string!(value)) }
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("value", self.value().fmt_res());
  }
}

impl RangeWidget {
  /// Get the value of the widget.
  pub fn value(&self) -> Result<f32> {
    unsafe { self.raw_value::<f32>() }
  }

  /// Set the value of the widget.
  pub fn set_value(&self, value: f32) -> Result<()> {
    unsafe { self.set_raw_value::<f32>(&value) }
  }

  /// Get the range and increment step of the widget.
  pub fn range_and_step(&self) -> Result<(RangeInclusive<f32>, f32)> {
    try_gp_internal!(gp_widget_get_range(self.as_ptr(), &out min, &out max, &out step));
    Ok((min..=max, step))
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    let (range, step) = match self.range_and_step() {
      Ok((range, step)) => (Some(range), Some(step)),
      Err(_) => (None, None),
    };
    f.field("range", &range).field("step", &step).field("value", &self.value().fmt_res());
  }
}

impl ToggleWidget {
  /// Check if the widget is toggled.
  pub fn is_toggled(&self) -> Result<Option<bool>> {
    unsafe { self.raw_value::<c_int>() }.map(|value| match value {
      0 => Some(false),
      1 => Some(true),
      _ => None,
    })
  }

  /// Set the toggled state of the widget.
  pub fn set_toggled(&self, value: bool) -> Result<()> {
    unsafe { self.set_raw_value::<c_int>(&(value as _)) }
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("toggled", self.is_toggled().fmt_res());
  }
}

impl RadioWidget {
  /// Get list of the available choices.
  pub fn choices(&self) -> Result<Vec<String>> {
    try_gp_internal!(let choice_count = gp_widget_count_choices(self.as_ptr()));

    (0..choice_count)
      .map(|i| {
        try_gp_internal!(gp_widget_get_choice(self.as_ptr(), i, &out choice));
        Ok(chars_to_string(choice))
      })
      .collect()
  }

  /// Get the current choice.
  pub fn choice(&self) -> Result<String> {
    Ok(chars_to_string(unsafe { self.raw_value::<*const c_char>()? }))
  }

  /// Set the current choice.
  pub fn set_choice(&self, value: &str) -> Result<()> {
    unsafe { self.set_raw_value::<c_char>(to_c_string!(value)) }
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("choices", &MaybeListFmt(|| self.choices())).field("choice", self.choice().fmt_res());
  }
}

impl DateWidget {
  /// Get the widget's value as a UNIX timestamp.
  pub fn timestamp(&self) -> Result<c_int> {
    unsafe { self.raw_value::<c_int>() }
  }

  /// Set the widget's value as a UNIX timestamp.
  pub fn set_timestamp(&self, value: c_int) -> Result<()> {
    unsafe { self.set_raw_value::<c_int>(&value) }
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("timestamp", self.timestamp().fmt_res());
  }
}

impl ButtonWidget {
  /// Press the button.
  pub fn press(&self, camera: &Camera) -> Result<()> {
    let callback = unsafe { self.raw_value::<libgphoto2_sys::CameraWidgetCallback>() }?
      .ok_or("Button without callback")?;
    let status = unsafe { callback(camera.camera, self.as_ptr(), camera.context) };
    if status < 0 {
      Err(Error::new(status, None))
    } else {
      Ok(())
    }
  }

  fn fmt_fields(&self, _f: &mut fmt::DebugStruct) {}
}

impl Widget {
  pub(crate) fn new_shared(widget: *mut libgphoto2_sys::CameraWidget) -> Result<Self> {
    unsafe {
      libgphoto2_sys::gp_widget_ref(widget);
    }

    Self::new_owned(widget)
  }
}
