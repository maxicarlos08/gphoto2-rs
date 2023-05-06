//! Camera configuration
//!
//! ## Configuring a camera
//! ```no_run
//! use gphoto2::{Context, widget::RadioWidget, Result};
//!
//! # fn main() -> Result<()> {
//! let context = Context::new()?;
//! let camera = context.autodetect_camera().wait()?;
//!
//! let mut config = camera.config_key::<RadioWidget>("iso").wait()?;
//! config.set_choice("100")?; // Set the iso to 100
//! camera.set_config(&config).wait(); // Apply setting to camera
//! # Ok(())
//! # }
//! ```

use crate::{
  helper::{as_ref, chars_to_string, to_c_string},
  task::{BackgroundPtr, Task},
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

impl Iterator for WidgetIterator<'_> {
  type Item = Widget;

  fn next(&mut self) -> Option<Self::Item> {
    self.range.next().map(|i| self.parent_widget.get_child(i).unwrap())
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.range.size_hint()
  }
}

impl ExactSizeIterator for WidgetIterator<'_> {
  fn len(&self) -> usize {
    self.range.len()
  }
}

/// Base widget type providing general information about the widget.
///
/// Normally you shouldn't use this type directly but should access its
/// properties via [`Widget`] or specific typed widgets instead.
pub struct WidgetBase {
  pub(crate) inner: BackgroundPtr<libgphoto2_sys::CameraWidget>,
}

impl Clone for WidgetBase {
  fn clone(&self) -> Self {
    try_gp_internal!(gp_widget_ref(*self.inner).unwrap());
    Self { inner: self.inner }
  }
}

impl Drop for WidgetBase {
  fn drop(&mut self) {
    let widget_ptr = self.inner;
    unsafe {
      Task::new(move || {
        try_gp_internal!(gp_widget_unref(*widget_ptr).unwrap());
      })
    }
    .background();
  }
}

impl WidgetBase {
  fn as_ptr(&self) -> *mut libgphoto2_sys::CameraWidget {
    *self.inner
  }

  /// Get exact widget type.
  fn ty(&self) -> libgphoto2_sys::CameraWidgetType {
    try_gp_internal!(gp_widget_get_type(*self.inner, &out widget_type).unwrap());
    widget_type
  }

  /// If true, the widget cannot be written
  pub fn readonly(&self) -> bool {
    try_gp_internal!(gp_widget_get_readonly(*self.inner, &out readonly).unwrap());
    readonly == 1
  }

  /// Get the widget label
  pub fn label(&self) -> String {
    try_gp_internal!(gp_widget_get_label(*self.inner, &out label).unwrap());
    chars_to_string(label)
  }

  /// Get the widget name
  pub fn name(&self) -> String {
    try_gp_internal!(gp_widget_get_name(*self.inner, &out name).unwrap());
    chars_to_string(name)
  }

  /// Get the widget id
  pub fn id(&self) -> i32 {
    try_gp_internal!(gp_widget_get_id(*self.inner, &out id).unwrap());
    id
  }

  /// Get information about the widget
  pub fn info(&self) -> String {
    try_gp_internal!(gp_widget_get_info(*self.inner, &out info).unwrap());
    chars_to_string(info)
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("id", &self.id())
      .field("name", &self.name())
      .field("label", &self.label())
      .field("readonly", &self.readonly());
  }

  unsafe fn raw_value<T>(&self) -> T {
    let mut value = std::mem::MaybeUninit::<T>::uninit();
    try_gp_internal!(gp_widget_get_value(*self.inner, value.as_mut_ptr().cast::<c_void>()).unwrap());
    value.assume_init()
  }

  unsafe fn set_raw_value<T>(&self, value: *const T) {
    try_gp_internal!(gp_widget_set_value(*self.inner, value.cast::<c_void>()).unwrap());
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

as_ref!(WidgetBase -> libgphoto2_sys::CameraWidget, **self.inner);

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
      pub(crate) fn new_owned(widget: BackgroundPtr<libgphoto2_sys::CameraWidget>) -> Self {
        let inner = WidgetBase { inner: widget };

        match inner.ty() {
          $($(libgphoto2_sys::CameraWidgetType::$gp_name)|+ => Widget::$variant($name { inner }),)*
        }
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
            _ => Err(Error::from(format!("Got {} but expected {:?}", stringify!($name),widget))),
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

/// Helper that prints `[_; count]` when using `{:?}` or the given list when using `{:#?}`.
struct MaybeListFmt<F>(F);

impl<Iter: IntoIterator, F: Fn() -> Iter> fmt::Debug for MaybeListFmt<F>
where
  Iter::IntoIter: ExactSizeIterator,
  Iter::Item: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let list = (self.0)();
    if f.alternate() {
      f.debug_list().entries(list).finish()
    } else {
      write!(f, "[_; {}]", list.into_iter().len())
    }
  }
}

impl GroupWidget {
  /// Creates a new [`WidgetIterator`]
  pub fn children_iter(&self) -> WidgetIterator<'_> {
    WidgetIterator { parent_widget: self, range: 0..self.children_count() }
  }

  /// Counts the children of the widget
  pub fn children_count(&self) -> usize {
    try_gp_internal!(let count = gp_widget_count_children(self.as_ptr()).unwrap());
    count.try_into().unwrap()
  }

  /// Gets a child by its index
  pub fn get_child(&self, index: usize) -> Result<Widget> {
    try_gp_internal!(gp_widget_get_child(self.as_ptr(), index.try_into()?, &out child)?);

    Ok(Widget::new_shared(BackgroundPtr(child)))
  }

  /// Get a child by its id
  pub fn get_child_by_id(&self, id: usize) -> Result<Widget> {
    try_gp_internal!(gp_widget_get_child_by_id(self.as_ptr(), id.try_into()?, &out child)?);

    Ok(Widget::new_shared(BackgroundPtr(child)))
  }

  /// Get a child by its label
  pub fn get_child_by_label(&self, label: &str) -> Result<Widget> {
    try_gp_internal!(gp_widget_get_child_by_label(self.as_ptr(), to_c_string!(label), &out child)?);

    Ok(Widget::new_shared(BackgroundPtr(child)))
  }

  /// Get a child by its name
  pub fn get_child_by_name(&self, name: &str) -> Result<Widget> {
    try_gp_internal!(gp_widget_get_child_by_name(self.as_ptr(), to_c_string!(name), &out child)?);

    Ok(Widget::new_shared(BackgroundPtr(child)))
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("children", &MaybeListFmt(|| self.children_iter()));
  }
}

impl TextWidget {
  /// Get the value of the widget.
  pub fn value(&self) -> String {
    chars_to_string(unsafe { self.raw_value::<*const c_char>() })
  }

  /// Set the value of the widget.
  pub fn set_value(&self, value: &str) -> Result<()> {
    unsafe {
      self.set_raw_value::<c_char>(to_c_string!(value));
    }
    Ok(())
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("value", &self.value());
  }
}

impl RangeWidget {
  /// Get the value of the widget.
  pub fn value(&self) -> f32 {
    unsafe { self.raw_value::<f32>() }
  }

  /// Set the value of the widget.
  pub fn set_value(&self, value: f32) {
    unsafe { self.set_raw_value::<f32>(&value) }
  }

  /// Get the range and increment step of the widget.
  pub fn range_and_step(&self) -> (RangeInclusive<f32>, f32) {
    try_gp_internal!(gp_widget_get_range(self.as_ptr(), &out min, &out max, &out step).unwrap());
    (min..=max, step)
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    let (range, step) = self.range_and_step();
    f.field("range", &range).field("step", &step).field("value", &self.value());
  }
}

impl ToggleWidget {
  /// Check if the widget is toggled.
  pub fn toggled(&self) -> Option<bool> {
    let value = unsafe { self.raw_value::<c_int>() };
    match value {
      0 => Some(false),
      1 => Some(true),
      _ => None,
    }
  }

  /// Set the toggled state of the widget.
  pub fn set_toggled(&self, value: bool) {
    unsafe { self.set_raw_value::<c_int>(&value.into()) }
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("toggled", &self.toggled());
  }
}

/// Iterator over the choices of a [`RadioWidget`].
pub struct ChoicesIter<'a> {
  widget: &'a RadioWidget,
  range: Range<c_int>,
}

impl<'a> Iterator for ChoicesIter<'a> {
  type Item = String;

  fn next(&mut self) -> Option<Self::Item> {
    self.range.next().map(|i| {
      try_gp_internal!(gp_widget_get_choice(self.widget.as_ptr(), i, &out choice).unwrap());
      chars_to_string(choice)
    })
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.range.size_hint()
  }
}

impl ExactSizeIterator for ChoicesIter<'_> {
  fn len(&self) -> usize {
    self.range.len()
  }
}

impl RadioWidget {
  /// Get list of the available choices.
  pub fn choices_iter(&self) -> ChoicesIter<'_> {
    try_gp_internal!(let choice_count = gp_widget_count_choices(self.as_ptr()).unwrap());
    ChoicesIter { widget: self, range: 0..choice_count }
  }

  /// Get the current choice.
  pub fn choice(&self) -> String {
    chars_to_string(unsafe { self.raw_value::<*const c_char>() })
  }

  /// Set the current choice.
  pub fn set_choice(&self, value: &str) -> Result<()> {
    unsafe {
      self.set_raw_value::<c_char>(to_c_string!(value));
    }
    Ok(())
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("choices", &MaybeListFmt(|| self.choices_iter())).field("choice", &self.choice());
  }
}

impl DateWidget {
  /// Get the widget's value as a UNIX timestamp.
  pub fn timestamp(&self) -> c_int {
    unsafe { self.raw_value::<c_int>() }
  }

  /// Set the widget's value as a UNIX timestamp.
  pub fn set_timestamp(&self, value: c_int) {
    unsafe { self.set_raw_value::<c_int>(&value) }
  }

  fn fmt_fields(&self, f: &mut fmt::DebugStruct) {
    f.field("timestamp", &self.timestamp());
  }
}

impl ButtonWidget {
  /// Press the button.
  pub fn press(&self, camera: &Camera) -> Result<()> {
    let callback = unsafe { self.raw_value::<libgphoto2_sys::CameraWidgetCallback>() }
      .ok_or("Button without callback")?;
    Error::check(unsafe { callback(*camera.camera, self.as_ptr(), *camera.context.inner) })?;
    Ok(())
  }

  fn fmt_fields(&self, _f: &mut fmt::DebugStruct) {}
}

impl Widget {
  pub(crate) fn new_shared(widget: BackgroundPtr<libgphoto2_sys::CameraWidget>) -> Self {
    try_gp_internal!(gp_widget_ref(*widget).unwrap());
    Self::new_owned(widget)
  }
}
