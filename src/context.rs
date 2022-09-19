//! Library context
use crate::{
  abilities::AbilitiesList, camera::Camera, list::CameraList, port::PortInfoList, try_gp_internal,
  Error, InnerPtr, Result,
};
use std::{ffi, marker::PhantomData};

/// Context used internally by gphoto
///
/// ## Example
///
/// ```no_run
/// use gphoto2::{Context, Result};
///
/// # fn main() -> Result<()> {
/// let context = Context::new()?;
///
/// let camera_list = context.list_cameras()?;
///
/// // Use first camera in the camera list
///
/// let (model, port) = &camera_list.to_vec()?[0];
/// context.get_camera(&model.to_string(), &port.to_string());
///
/// # Ok(())
/// # }
///
/// ```
pub struct Context<'a> {
  pub(crate) inner: *mut libgphoto2_sys::GPContext,
  _phantom: PhantomData<&'a libgphoto2_sys::GPContext>,
}

impl Drop for Context<'_> {
  fn drop(&mut self) {
    unsafe { libgphoto2_sys::gp_context_unref(self.inner) }
  }
}

impl<'a> InnerPtr<'a, libgphoto2_sys::GPContext> for Context<'a> {
  unsafe fn inner_mut_ptr(&'a self) -> &'a *mut libgphoto2_sys::GPContext {
    &self.inner
  }
}

impl<'a> Context<'a> {
  /// Create a new context
  pub fn new() -> Result<Self> {
    let context_ptr = unsafe { libgphoto2_sys::gp_context_new() };

    if context_ptr.is_null() {
      Err(Error::new(libgphoto2_sys::GP_ERROR_NO_MEMORY))
    } else {
      Ok(Self { inner: context_ptr, _phantom: PhantomData })
    }
  }

  /// Lists all available cameras and their ports
  ///
  /// Returns a list of (camera_name, port_path)
  /// which can be used in [`Context::get_camera`].
  pub fn list_cameras(&self) -> Result<CameraList> {
    let camera_list = CameraList::new()?;

    try_gp_internal!(gp_camera_autodetect(camera_list.inner, self.inner));

    Ok(camera_list)
  }

  /// Auto chooses a camera
  ///
  /// ```no_run
  /// use gphoto2::{Context, Result};
  ///
  /// # fn main() -> Result<()> {
  /// let context = Context::new()?;
  /// if let Ok(camera) = context.autodetect_camera() {
  ///   println!("Successfully autodetected camera '{}'", camera.abilities()?.model());
  /// } else {
  ///   println!("Could not autodetect camera");
  /// }
  /// # Ok(())
  /// # }
  /// ```
  pub fn autodetect_camera(&self) -> Result<Camera> {
    try_gp_internal!(gp_camera_new(&out camera_ptr));
    try_gp_internal!(gp_camera_init(camera_ptr, self.inner));

    Ok(Camera::new(camera_ptr, self))
  }

  /// Initialize a camera knowing its model name and port path
  ///
  /// ```no_run
  /// use gphoto2::{Context, Result};
  ///
  /// # fn main() -> Result<()> {
  /// let context = Context::new()?;
  /// let camera_list = context.list_cameras()?;
  /// let camera_list = camera_list.to_vec()?;
  ///
  /// if camera_list.len() < 1 {
  ///   Err("No cameras found")?
  /// }
  ///
  /// let camera = context.get_camera(&camera_list[0].0[..], &camera_list[0].1[..])?;
  /// # Ok(())
  /// # }
  pub fn get_camera(&self, model: &str, port_path: &str) -> Result<Camera> {
    let abilities_list = AbilitiesList::new(self)?;
    let port_info_list = PortInfoList::new()?;

    try_gp_internal!(gp_camera_new(&out camera));

    try_gp_internal!(let model_index = gp_abilities_list_lookup_model(
      abilities_list.inner,
      ffi::CString::new(model)?.as_ptr(),
    ));

    try_gp_internal!(gp_abilities_list_get_abilities(
      abilities_list.inner,
      model_index,
      &out model_abilities
    ));
    try_gp_internal!(gp_camera_set_abilities(camera, model_abilities));

    try_gp_internal!(let p = gp_port_info_list_lookup_path(
      port_info_list.inner,
      ffi::CString::new(port_path)?.as_ptr()
    ));
    let port_info = port_info_list.get_port_info(p)?;
    try_gp_internal!(gp_camera_set_port_info(camera, port_info.inner));

    Ok(Camera::new(camera, self))
  }
}
