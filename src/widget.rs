//! Camera configuration
//!
//! ## Configuring a camera
//! ```no_run
//! use gphoto2::{Context, widget::WidgetValue, Result};
//!
//! # fn main() -> Result<()> {
//! let context = Context::new()?;
//! let camera = context.autodetect_camera()?;
//!
//! let mut config = camera.config_key("iso")?;
//! config.set_value(WidgetValue::Menu("100".to_string()))?; // Set the iso to 100
//! camera.set_config(&config); // Apply setting to camera
//! # Ok(())
//! # }
//! ```

use crate::{
  helper::{chars_to_cow, to_c_string},
  try_gp_internal, InnerPtr, Result,
};
use std::{
  borrow::Cow,
  ffi, fmt,
  marker::PhantomData,
  os::raw::{c_int, c_void},
};

/// Value of a widget
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone)]
pub enum WidgetType {
  /// Root configuration object
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
pub struct WidgetIterator<'a> {
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

impl fmt::Debug for Widget<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Widget")
      .field("id", &self.id().ok())
      .field("name", &self.name().ok())
      .field("label", &self.label().ok())
      .field("readonly", &self.readonly().ok())
      .field("widget_type", &self.widget_type().ok())
      .field(
        "value",
        &match self.value() {
          Ok((Some(value), _)) => Some(value),
          _ => None,
        },
      )
      .field("children", &self.children_iter().map(|iter| iter.collect::<Vec<Widget>>()))
      .finish()
  }
}

impl<'a> InnerPtr<'a, libgphoto2_sys::CameraWidget> for Widget<'a> {
  unsafe fn inner_mut_ptr(&'a self) -> &'a *mut libgphoto2_sys::CameraWidget {
    &self.inner
  }
}

impl<'a> Widget<'a> {
  pub(crate) fn new(widget: *mut libgphoto2_sys::CameraWidget) -> Self {
    unsafe { libgphoto2_sys::gp_widget_ref(widget) };

    Self { inner: widget, _phantom: PhantomData }
  }

  /// If true, the widget cannot be written
  pub fn readonly(&self) -> Result<bool> {
    try_gp_internal!(gp_widget_get_readonly(self.inner, &out readonly));

    Ok(readonly == 1)
  }

  /// Get the widget label
  pub fn label(&self) -> Result<Cow<str>> {
    try_gp_internal!(gp_widget_get_label(self.inner, &out label));

    Ok(chars_to_cow(label))
  }

  /// Get the widget name
  pub fn name(&self) -> Result<Cow<str>> {
    try_gp_internal!(gp_widget_get_name(self.inner, &out name));
    Ok(chars_to_cow(name))
  }

  /// Get the widget id
  pub fn id(&self) -> Result<i32> {
    try_gp_internal!(gp_widget_get_id(self.inner, &out id));

    Ok(id)
  }

  /// Get information about the widget
  pub fn info(&self) -> Result<Cow<str>> {
    try_gp_internal!(gp_widget_get_info(self.inner, &out info));

    Ok(chars_to_cow(info))
  }

  /// Creates a new [`WidgetIterator`]
  pub fn children_iter(&'a self) -> Result<WidgetIterator<'a>> {
    Ok(WidgetIterator { parent_widget: self, count: self.children_count()?, index: 0 })
  }

  /// Counts the children of the widget
  pub fn children_count(&self) -> Result<usize> {
    try_gp_internal!(let count = gp_widget_count_children(self.inner));
    Ok(count as usize)
  }

  /// Gets a child by its index
  pub fn get_child(&self, index: usize) -> Result<Widget<'a>> {
    try_gp_internal!(gp_widget_get_child(self.inner, index as c_int, &out child));

    Ok(Self::new(child))
  }

  /// Get a child by its id
  pub fn get_child_by_id(&self, id: usize) -> Result<Widget<'a>> {
    try_gp_internal!(gp_widget_get_child_by_id(self.inner, id as c_int, &out child));

    Ok(Self::new(child))
  }

  /// Get a child by its label
  pub fn get_child_by_label(&self, label: &str) -> Result<Widget<'a>> {
    try_gp_internal!(gp_widget_get_child_by_label(self.inner, to_c_string!(label), &out child));

    Ok(Self::new(child))
  }

  /// Get a child by its name
  pub fn get_child_by_name(&self, name: &str) -> Result<Widget<'a>> {
    try_gp_internal!(gp_widget_get_child_by_name(self.inner, to_c_string!(name), &out child));

    Ok(Self::new(child))
  }

  /// Get the type of the widget
  pub fn widget_type(&self) -> Result<WidgetType> {
    use libgphoto2_sys::CameraWidgetType;

    try_gp_internal!(gp_widget_get_type(self.inner, &out widget_type));

    Ok(match widget_type {
      CameraWidgetType::GP_WIDGET_WINDOW => WidgetType::Window,
      CameraWidgetType::GP_WIDGET_SECTION => WidgetType::Section,
      CameraWidgetType::GP_WIDGET_TEXT => WidgetType::Text,
      CameraWidgetType::GP_WIDGET_RANGE => {
        try_gp_internal!(gp_widget_get_range(self.inner, &out min, &out max, &out increment));

        WidgetType::Range { min, max, increment }
      }
      CameraWidgetType::GP_WIDGET_TOGGLE => WidgetType::Toggle,
      CameraWidgetType::GP_WIDGET_MENU | CameraWidgetType::GP_WIDGET_RADIO => {
        try_gp_internal!(let choice_count = gp_widget_count_choices(self.inner));
        let mut choices = Vec::with_capacity(choice_count as usize);

        for choice_i in 0..choice_count {
          try_gp_internal!(gp_widget_get_choice(self.inner, choice_i, &out choice));

          choices.push(chars_to_cow(choice).to_string());
        }

        WidgetType::Menu { choices, radio: widget_type == CameraWidgetType::GP_WIDGET_RADIO }
      }
      CameraWidgetType::GP_WIDGET_BUTTON => WidgetType::Button,
      CameraWidgetType::GP_WIDGET_DATE => WidgetType::Date,
    })
  }

  fn raw_value<T>(&self) -> Result<T> {
    try_gp_internal!(gp_widget_get_value(self.inner, &out value as *mut T as *mut c_void));
    Ok(value)
  }

  fn str_value(&self) -> Result<String> {
    Ok(chars_to_cow(self.raw_value()?).into_owned())
  }

  /// Get the widget value and type
  pub fn value(&self) -> Result<(Option<WidgetValue>, WidgetType)> {
    let widget_type = self.widget_type()?;

    Ok((
      match widget_type {
        WidgetType::Window | WidgetType::Button | WidgetType::Section => None,
        WidgetType::Text => Some(WidgetValue::Text(self.str_value()?)),
        WidgetType::Range { .. } => Some(WidgetValue::Range(self.raw_value()?)),
        WidgetType::Toggle => Some(WidgetValue::Toggle(self.raw_value::<c_int>()? == 0)),
        WidgetType::Date => Some(WidgetValue::Date(self.raw_value()?)),
        WidgetType::Menu { .. } => Some(WidgetValue::Menu(self.str_value()?)),
      },
      widget_type,
    ))
  }

  /// Sets the value of the widget
  ///
  /// **Note**: This only sets the value of the configuration, to apply the setting to the camera use [`Camera::set_config`](crate::Camera::set_config)
  pub fn set_value(&mut self, value: WidgetValue) -> Result<()> {
    let self_type = self.widget_type()?;

    match self_type {
      WidgetType::Window => Err("Window has no value")?,
      WidgetType::Section => Err("Section has no value")?,
      WidgetType::Button => Err("Button has no value")?,
      WidgetType::Text => {
        if let WidgetValue::Text(text) = value {
          try_gp_internal!(gp_widget_set_value(self.inner, to_c_string!(text).cast::<c_void>()));
        } else {
          Err("Expected value to be a string")?;
        }
      }
      WidgetType::Range { min, max, .. } => {
        if let WidgetValue::Range(range_value) = value {
          if (range_value < min) || (range_value > max) {
            Err("Value out of range")?;
          }

          try_gp_internal!(gp_widget_set_value(
            self.inner,
            &range_value as *const f32 as *const c_void
          ));
        } else {
          Err("Expected value to be Range")?;
        }
      }
      WidgetType::Toggle => {
        if let WidgetValue::Toggle(toggle_value) = value {
          let toggle_value = if toggle_value { 1 } else { 0 };
          try_gp_internal!(gp_widget_set_value(
            self.inner,
            &toggle_value as *const c_int as *const c_void
          ));
        } else {
          Err("Expected value to be Toggle")?;
        }
      }
      WidgetType::Date => {
        if let WidgetValue::Date(unix_date) = value {
          try_gp_internal!(gp_widget_set_value(
            self.inner,
            &unix_date as *const c_int as *const c_void
          ));
        } else {
          Err("Expected value to be Date")?;
        }
      }
      WidgetType::Menu { choices, .. } => {
        if let WidgetValue::Menu(choice) = value {
          if !choices.contains(&choice) {
            Err("Choice not in choices")?;
          }

          try_gp_internal!(gp_widget_set_value(self.inner, to_c_string!(choice).cast::<c_void>()));
        } else {
          Err("Expected value to be Menu")?;
        }
      }
    }

    Ok(())
  }
}

impl<'a> Iterator for WidgetIterator<'a> {
  type Item = Widget<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.index >= self.count {
      None
    } else {
      let child = self.parent_widget.get_child(self.index).ok();
      self.index += 1;

      child
    }
  }
}
