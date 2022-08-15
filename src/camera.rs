//! Camera related stuff

use crate::{
  abilities::Abilities, file::CameraFilePath, helper::camera_text_to_str, try_gp_internal, Result,
};
use std::mem::MaybeUninit;

/// Represents a camera
pub struct Camera {
  pub(crate) camera: *mut libgphoto2_sys::Camera,
  pub(crate) context: *mut libgphoto2_sys::GPContext,
}

impl Drop for Camera {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_camera_unref(self.camera);
      libgphoto2_sys::gp_context_unref(self.context);
    }
  }
}

impl Camera {
  /// Capture image
  ///
  /// ## Returns
  ///
  /// A [`CameraFilePath`] which can be downloaded to the host system
  // TODO: Usage example
  pub fn capture_image(&self) -> Result<CameraFilePath> {
    let mut file_path_ptr = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_camera_capture(
      self.camera,
      libgphoto2_sys::CameraCaptureType::GP_CAPTURE_IMAGE,
      &mut file_path_ptr,
      self.context
    ))?;

    Ok(file_path_ptr.into())
  }

  /// Get the camera's [`Abilities`]
  pub fn abilities(&self) -> Result<Abilities> {
    let mut abilities = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_abilities(self.camera, &mut abilities))?;

    Ok(abilities.into())
  }


  /// Summary of the cameras model, settings, capabilities, etc.
  pub fn summary(&self) -> Result<String> {
    let mut summary = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_summary(
      self.camera,
      &mut summary,
      self.context
    ))?;

    Ok(camera_text_to_str(summary).to_string())
  }

  // TODO: settings, port, summary (manual?, driver?)
}
