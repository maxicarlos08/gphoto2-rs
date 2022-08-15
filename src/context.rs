use crate::{
  abilities::AbilitiesList, camera::Camera, list::CameraList, port::PortInfoList, try_gp_internal,
  Error, Result,
};
use std::{ffi, mem::MaybeUninit};

pub struct Context {
  pub(crate) inner: *mut libgphoto2_sys::GPContext,
}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe { libgphoto2_sys::gp_context_unref(self.inner) }
  }
}

impl Context {
  pub fn new() -> Result<Self> {
    let context_ptr = unsafe { libgphoto2_sys::gp_context_new() };

    if context_ptr.is_null() {
      Err(Error::new(libgphoto2_sys::GP_ERROR_NO_MEMORY))
    } else {
      Ok(Self { inner: context_ptr })
    }
  }

  pub fn list_cameras(&self) -> Result<CameraList> {
    let camera_list = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_camera_autodetect(camera_list, self.inner))?;

    Ok(camera_list.into())
  }

  /// Auto chooses a camera
  pub fn autodetect_camera(&self) -> Result<Camera> {
    let mut camera_ptr = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_camera_new(&mut camera_ptr))?;
    try_gp_internal!(libgphoto2_sys::gp_camera_init(camera_ptr, self.inner))?;

    Ok(Camera { camera: camera_ptr, context: self.inner })
  }

  pub fn get_camera(&self, model: &str, port: &str) -> Result<Camera> {
    let mut model_abilities = unsafe { MaybeUninit::zeroed().assume_init() };
    let mut camera = unsafe { MaybeUninit::zeroed().assume_init() };
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

    Ok(Camera { camera, context: self.inner })
  }
}
