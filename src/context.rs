//! Library context
use crate::{
  abilities::AbilitiesList, camera::Camera, helper::uninit, list::CameraList, port::PortInfoList,
  try_gp_internal, Error, Result,
};
use std::ffi;

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
pub struct Context {
  pub(crate) inner: *mut libgphoto2_sys::GPContext,
}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe { libgphoto2_sys::gp_context_unref(self.inner) }
  }
}

impl Context {
  /// Create a new context
  pub fn new() -> Result<Self> {
    let context_ptr = unsafe { libgphoto2_sys::gp_context_new() };

    if context_ptr.is_null() {
      Err(Error::new(libgphoto2_sys::GP_ERROR_NO_MEMORY))
    } else {
      Ok(Self { inner: context_ptr })
    }
  }

  /// Lists all available cameras and their ports
  pub fn list_cameras(&self) -> Result<CameraList> {
    let camera_list = CameraList::new()?;

    try_gp_internal!(libgphoto2_sys::gp_camera_autodetect(camera_list.inner, self.inner))?;

    Ok(camera_list)
  }

  /// Auto chooses a camera
  pub fn autodetect_camera(&self) -> Result<Camera> {
    let mut camera_ptr = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_new(&mut camera_ptr))?;
    try_gp_internal!(libgphoto2_sys::gp_camera_init(camera_ptr, self.inner))?;

    Ok(Camera::new(camera_ptr, self.inner))
  }

  /// Initialize a camera knowing its model name and port
  pub fn get_camera(&self, model: &str, port: &str) -> Result<Camera> {
    let mut model_abilities = unsafe { uninit() };
    let mut camera = unsafe { uninit() };
    let abilities_list = AbilitiesList::new(self)?;
    let port_info_list = PortInfoList::new()?;

    try_gp_internal!(libgphoto2_sys::gp_camera_new(&mut camera))?;

    let model_index = try_gp_internal!(libgphoto2_sys::gp_abilities_list_lookup_model(
      abilities_list.inner,
      ffi::CString::new(model)?.as_ptr(),
    ))?;

    try_gp_internal!(libgphoto2_sys::gp_abilities_list_get_abilities(
      abilities_list.inner,
      model_index,
      &mut model_abilities
    ))?;
    try_gp_internal!(libgphoto2_sys::gp_camera_set_abilities(camera, model_abilities))?;

    let p = try_gp_internal!(libgphoto2_sys::gp_port_info_list_lookup_path(
      port_info_list.inner,
      ffi::CString::new(port)?.as_ptr()
    ))?;
    let port_info = port_info_list.get_port_info(p)?;
    try_gp_internal!(libgphoto2_sys::gp_camera_set_port_info(camera, port_info.inner))?;

    Ok(Camera::new(camera, self.inner))
  }
}
