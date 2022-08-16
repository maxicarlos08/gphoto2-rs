//! Widgets are used to configure the camera

use crate::{
  helper::{chars_to_cow, uninit},
  try_gp_internal, Result,
};
use std::{
  borrow::Cow,
  ffi,
  marker::PhantomData,
  os::raw::{c_char, c_float, c_int, c_void},
};

macro_rules! get_widget_value {
  ($widget:expr, $tp:ty) => {{
    let value = unsafe { $crate::helper::uninit() };
    $crate::try_gp_internal!(libgphoto2_sys::gp_widget_get_value($widget, value))?;
    unsafe { *(value as *mut $tp) }
  }};
}

/// Value of a widget
pub enum WidgetValue {
  /// Textual data
  Text(String),
  /// Float in a range
  Range(f32),
  /// Boolean
  Toggle(bool),
  /// Selected choice
  Menu(String),
  /// Date
  Date(c_int),
}

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
    min: f32,
    /// Maximum value
    max: f32,
    /// Step
    increment: f32,
  },
  /// Boolean
  Toggle,
  /// Choice between many values
  Menu {
    /// Choices
    choices: Vec<String>,
    /// If the value was internally represented as radio (which is the same)
    radio: bool,
  },
  /// Button
  Button,
  /// Date
  Date,
}

/// Iterator over the children of a widget
pub struct WidgetChildrenIter<'a> {
  parent_widget: &'a Widget<'a>,
  count: usize,
  index: usize,
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

  /// If true, the widget cannot be written
  pub fn readonly(&self) -> Result<bool> {
    let mut readonly = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_readonly(self.inner, &mut readonly))?;

    Ok(readonly == 1)
  }

  /// Get the widget label
  pub fn label(&self) -> Result<Cow<str>> {
    let mut label = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_label(self.inner, &mut label))?;

    Ok(chars_to_cow(label))
  }

  /// Get the widget name
  pub fn name(&self) -> Result<Cow<str>> {
    let mut name = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_name(self.inner, &mut name))?;
    Ok(chars_to_cow(name))
  }

  /// Get the widget id
  pub fn id(&self) -> Result<i32> {
    let mut id = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_id(self.inner, &mut id))?;

    Ok(id)
  }

  /// Get information about the widget
  pub fn info(&self) -> Result<Cow<str>> {
    let mut info = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_info(self.inner, &mut info))?;

    Ok(chars_to_cow(info))
  }

  /// Creates a new [`WidgetChildrenIter`]
  pub fn children_iter(&'a self) -> Result<WidgetChildrenIter<'a>> {
    Ok(WidgetChildrenIter { parent_widget: self, count: self.children_count()?, index: 0 })
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

  /// Get the type of the widget
  pub fn widget_type(&self) -> Result<WidgetType> {
    use libgphoto2_sys::CameraWidgetType;

    let mut widget_type = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_widget_get_type(self.inner, &mut widget_type))?;

    Ok(match widget_type {
      CameraWidgetType::GP_WIDGET_WINDOW => WidgetType::Window,
      CameraWidgetType::GP_WIDGET_SECTION => WidgetType::Section,
      CameraWidgetType::GP_WIDGET_TEXT => WidgetType::Text,
      CameraWidgetType::GP_WIDGET_RANGE => {
        let (mut min, mut max, mut increment) = unsafe { (uninit(), uninit(), uninit()) };

        try_gp_internal!(libgphoto2_sys::gp_widget_get_range(
          self.inner,
          &mut min,
          &mut max,
          &mut increment
        ))?;

        WidgetType::Range { min, max, increment }
      }
      CameraWidgetType::GP_WIDGET_TOGGLE => WidgetType::Toggle,
      CameraWidgetType::GP_WIDGET_MENU | CameraWidgetType::GP_WIDGET_RADIO => {
        let choice_count = try_gp_internal!(libgphoto2_sys::gp_widget_count_choices(self.inner))?;
        let mut choices = Vec::with_capacity(choice_count as usize);

        for choice_i in 0..choice_count {
          let mut choice = unsafe { uninit() };

          try_gp_internal!(libgphoto2_sys::gp_widget_get_choice(
            self.inner,
            choice_i,
            &mut choice
          ))?;

          choices.push(chars_to_cow(choice).to_string());
        }

        WidgetType::Menu {
          choices: choices,
          radio: widget_type == CameraWidgetType::GP_WIDGET_RADIO,
        }
      }
      CameraWidgetType::GP_WIDGET_BUTTON => WidgetType::Button,
      CameraWidgetType::GP_WIDGET_DATE => WidgetType::Date,
    })
  }

  /// Get the widget value and type
  pub fn get_value(&self) -> Result<(Option<WidgetValue>, WidgetType)> {
    let widget_type = self.widget_type()?;

    Ok((
      match widget_type {
        WidgetType::Window | WidgetType::Button | WidgetType::Section => None,
        WidgetType::Text => {
          let text = chars_to_cow(get_widget_value!(self.inner, *const c_char));

          Some(WidgetValue::Text(text.to_string()))
        }
        WidgetType::Range { .. } => {
          let range_value = get_widget_value!(self.inner, c_float);

          Some(WidgetValue::Range(range_value))
        }
        WidgetType::Toggle => {
          let boolean = get_widget_value!(self.inner, c_int);

          Some(WidgetValue::Toggle(boolean == 0))
        }
        WidgetType::Date => {
          let date_int = get_widget_value!(self.inner, c_int);

          Some(WidgetValue::Date(date_int))
        }
        WidgetType::Menu { .. } => {
          let choice = chars_to_cow(get_widget_value!(self.inner, *const c_char));

          Some(WidgetValue::Menu(choice.to_string()))
        }
      },
      widget_type,
    ))
  }

  /// Sets the value of the widget
  pub fn set_value(&mut self, value: WidgetValue) -> Result<()> {
    let self_type = self.widget_type()?;

    match self_type {
      WidgetType::Window => Err("Window has no value")?,
      WidgetType::Section => Err("Section has no value")?,
      WidgetType::Button => Err("Button has no value")?,
      WidgetType::Text => {
        if let WidgetValue::Text(text) = value {
          try_gp_internal!(libgphoto2_sys::gp_widget_set_value(
            self.inner,
            text.as_ptr() as *const c_void
          ))?;
        } else {
          Err("Expected value to be a string")?;
        }
      }
      WidgetType::Range { min, max, .. } => {
        if let WidgetValue::Range(range_value) = value {
          if (range_value < min) || (range_value > max) {
            Err("Value out of range")?;
          }

          try_gp_internal!(libgphoto2_sys::gp_widget_set_value(
            self.inner,
            &range_value as *const f32 as *const c_void
          ))?;
        } else {
          Err("Expected value to be Range")?;
        }
      }
      WidgetType::Toggle => {
        if let WidgetValue::Toggle(toggle_value) = value {
          let toggle_value = if toggle_value { 1 } else { 0 };
          try_gp_internal!(libgphoto2_sys::gp_widget_set_value(
            self.inner,
            &toggle_value as *const c_int as *const c_void
          ))?;
        } else {
          Err("Expected value to be Toggle")?;
        }
      }
      WidgetType::Date => {
        if let WidgetValue::Date(unix_date) = value {
          try_gp_internal!(libgphoto2_sys::gp_widget_set_value(
            self.inner,
            &unix_date as *const c_int as *const c_void
          ))?;
        } else {
          Err("Expected value to be Date")?;
        }
      }
      WidgetType::Menu { choices, .. } => {
        if let WidgetValue::Menu(choice) = value {
          if !choices.contains(&choice) {
            Err("Choice not in choices")?;
          }

          try_gp_internal!(libgphoto2_sys::gp_widget_set_value(
            self.inner,
            choice.as_ptr() as *const c_void
          ))?;
        } else {
          Err("Expected value to be Menu")?;
        }
      }
    }

    Ok(())
  }
}

impl<'a> Iterator for WidgetChildrenIter<'a> {
  type Item = Widget<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.count >= self.index {
      None
    } else {
      let child = self.parent_widget.get_child(self.index).ok();
      self.index += 1;

      child
    }
  }
}
