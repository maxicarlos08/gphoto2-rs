//! Gphoto device abilities

use crate::{context::Context, try_gp_internal, Result};
use std::{borrow::Cow, ffi, marker::PhantomData, mem::MaybeUninit};

pub(crate) struct AbilitiesList<'a> {
  pub(crate) inner: *mut libgphoto2_sys::CameraAbilitiesList,
  _phantom: PhantomData<&'a ffi::c_void>,
}

/// Provides functions to get device abilities
pub struct Abilities {
  pub(crate) inner: libgphoto2_sys::CameraAbilities,
}

/// Status of the gphoto driver used
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CameraDriverStatus {
  /// The driver is stable
  Production,
  /// Driver is being tested, should be more stable than [`Experimental`](CameraDriverStatus::Experimental)
  Testing,
  /// (Unstable) Experimental driver, might not work as expected
  Experimental,
  /// The driver is deprecated, don't use this
  Deprecated,
}

/// Type of the device
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DeviceType {
  /// Still camera
  Camera,
  /// MTP audio device
  AudioPlayer,
}

/// Available operations on the camera
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CameraOperations(i32);

/// Available operations on files
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FileOperations(i32);

/// Available operations of folders
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FolderOperations(i32);

impl Drop for AbilitiesList<'_> {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_abilities_list_free(self.inner);
    }
  }
}

impl From<libgphoto2_sys::CameraOperation> for CameraOperations {
  fn from(op: libgphoto2_sys::CameraOperation) -> Self {
    Self(op as i32)
  }
}

impl From<libgphoto2_sys::CameraFileOperation> for FileOperations {
  fn from(op: libgphoto2_sys::CameraFileOperation) -> Self {
    Self(op as i32)
  }
}

impl From<libgphoto2_sys::CameraFolderOperation> for FolderOperations {
  fn from(op: libgphoto2_sys::CameraFolderOperation) -> Self {
    Self(op as i32)
  }
}

impl From<libgphoto2_sys::GphotoDeviceType> for DeviceType {
  fn from(device_type: libgphoto2_sys::GphotoDeviceType) -> Self {
    use libgphoto2_sys::GphotoDeviceType;

    match device_type {
      GphotoDeviceType::GP_DEVICE_STILL_CAMERA => Self::Camera,
      GphotoDeviceType::GP_DEVICE_AUDIO_PLAYER => Self::AudioPlayer,
    }
  }
}

impl From<libgphoto2_sys::CameraDriverStatus> for CameraDriverStatus {
  fn from(status: libgphoto2_sys::CameraDriverStatus) -> Self {
    use libgphoto2_sys::CameraDriverStatus as GPDriverStatus;
    match status {
      GPDriverStatus::GP_DRIVER_STATUS_PRODUCTION => Self::Production,
      GPDriverStatus::GP_DRIVER_STATUS_TESTING => Self::Testing,
      GPDriverStatus::GP_DRIVER_STATUS_EXPERIMENTAL => Self::Experimental,
      GPDriverStatus::GP_DRIVER_STATUS_DEPRECATED => Self::Deprecated,
    }
  }
}

impl From<libgphoto2_sys::CameraAbilities> for Abilities {
  fn from(abilities: libgphoto2_sys::CameraAbilities) -> Self {
    Self { inner: abilities }
  }
}

impl<'a> AbilitiesList<'a> {
  pub(crate) fn new(context: &Context) -> Result<Self> {
    let mut abilities_inner = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_abilities_list_new(&mut abilities_inner))?;
    try_gp_internal!(libgphoto2_sys::gp_abilities_list_load(abilities_inner, context.inner))?;

    Ok(Self { inner: abilities_inner, _phantom: PhantomData })
  }
}

impl Abilities {
  /// Get the model of the camera
  pub fn model(&self) -> Cow<str> {
    unsafe { String::from_utf8_lossy(ffi::CStr::from_ptr(self.inner.model.as_ptr()).to_bytes()) }
  }

  /// Get the [driver status](CameraDriverStatus) of the device
  pub fn driver_status(&self) -> CameraDriverStatus {
    self.inner.status.into()
  }

  // TODO: Port, Port speeds
  // TODO: Usb info

  /// Get the [camera operations](CameraOperations) of the device
  pub fn camera_operations(&self) -> CameraOperations {
    self.inner.operations.into()
  }

  /// Get the [file operations](FileOperations) of the device
  pub fn file_operations(&self) -> FileOperations {
    self.inner.file_operations.into()
  }

  /// Get the [folder operations](FolderOperations) of the device
  pub fn folder_operations(&self) -> FolderOperations {
    self.inner.folder_operations.into()
  }

  /// Get the [device type](DeviceType) of the device
  pub fn device_type(&self) -> DeviceType {
    self.inner.device_type.into()
  }
}

macro_rules! impl_bitmask_check {
  ($name:ident, $op:expr) => {
    #[doc = "Check for the \"$name\" ability"]
    #[inline]
    pub fn $name(&self) -> bool {
      self.0 & $op as i32 != 0
    }
  };
}

impl CameraOperations {
  impl_bitmask_check!(capture_image, libgphoto2_sys::CameraOperation::GP_OPERATION_CAPTURE_IMAGE);

  impl_bitmask_check!(capture_video, libgphoto2_sys::CameraOperation::GP_OPERATION_CAPTURE_VIDEO);

  impl_bitmask_check!(capture_audio, libgphoto2_sys::CameraOperation::GP_OPERATION_CAPTURE_AUDIO);

  impl_bitmask_check!(
    capture_preview,
    libgphoto2_sys::CameraOperation::GP_OPERATION_CAPTURE_PREVIEW
  );

  impl_bitmask_check!(config, libgphoto2_sys::CameraOperation::GP_OPERATION_CONFIG);

  impl_bitmask_check!(
    trigger_capture,
    libgphoto2_sys::CameraOperation::GP_OPERATION_TRIGGER_CAPTURE
  );
}

impl FileOperations {
  impl_bitmask_check!(delete, libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_DELETE);

  impl_bitmask_check!(preview, libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_PREVIEW);

  impl_bitmask_check!(raw, libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_RAW);

  impl_bitmask_check!(audio, libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_AUDIO);

  impl_bitmask_check!(exif, libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_EXIF);
}

impl FolderOperations {
  impl_bitmask_check!(
    delete_all,
    libgphoto2_sys::CameraFolderOperation::GP_FOLDER_OPERATION_DELETE_ALL
  );

  impl_bitmask_check!(
    put_file,
    libgphoto2_sys::CameraFolderOperation::GP_FOLDER_OPERATION_PUT_FILE
  );

  impl_bitmask_check!(
    make_dir,
    libgphoto2_sys::CameraFolderOperation::GP_FOLDER_OPERATION_MAKE_DIR
  );

  impl_bitmask_check!(
    remove_dir,
    libgphoto2_sys::CameraFolderOperation::GP_FOLDER_OPERATION_REMOVE_DIR
  );
}
