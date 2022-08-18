//! Device abilities
//!
//! The device abilities describe the abilities of the driver used to connect to a device.

use crate::{
  context::Context,
  helper::{chars_to_cow, uninit},
  try_gp_internal, Result,
};
use std::{borrow::Cow, ffi, fmt, marker::PhantomData, os::raw::c_int};

pub(crate) struct AbilitiesList<'a> {
  pub(crate) inner: *mut libgphoto2_sys::CameraAbilitiesList,
  _phantom: PhantomData<&'a ffi::c_void>,
}

/// Provides functions to get device abilities
///
/// ## Abilities
///  - [`id`](Abilities::id): Camera ID
///  - [`model`](Abilities::model): Camera model
///  - [`driver_status`](Abilities::driver_status): Status of the camera driver
///  - [`camera_operations`](Abilities::camera_operations): Available operations on the camera
///  - [`file_operations`](Abilities::file_operations): Available operations on files
///  - [`folder_operations`](Abilities::folder_operations): Available operations on folder
///  - [`device_type`](Abilities::device_type): Type of the device
pub struct Abilities {
  pub(crate) inner: libgphoto2_sys::CameraAbilities,
}

/// Camera USB information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbInfo {
  /// Vendor ID
  pub vendor: usize,
  /// Product ID
  pub product: usize,
  /// Device class
  pub class: usize,
  /// Device subclass
  pub subclass: usize,
  /// Device protocol
  pub protocol: usize,
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

// TODO: Better debug implementation for CameraOperations, FileOperations and FolderOperations

/// Available operations on the camera
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CameraOperations(c_int);

/// Available operations on files
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FileOperations(c_int);

/// Available operations of folders
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FolderOperations(c_int);

impl Drop for AbilitiesList<'_> {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_abilities_list_free(self.inner);
    }
  }
}

impl From<libgphoto2_sys::CameraOperation> for CameraOperations {
  fn from(op: libgphoto2_sys::CameraOperation) -> Self {
    Self(op as c_int)
  }
}

impl From<libgphoto2_sys::CameraFileOperation> for FileOperations {
  fn from(op: libgphoto2_sys::CameraFileOperation) -> Self {
    Self(op as c_int)
  }
}

impl From<libgphoto2_sys::CameraFolderOperation> for FolderOperations {
  fn from(op: libgphoto2_sys::CameraFolderOperation) -> Self {
    Self(op as c_int)
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

impl fmt::Debug for Abilities {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Abilities")
      .field("id", &self.id())
      .field("model", &self.model())
      .field("driver_status", &self.driver_status())
      .field("camera_operations", &self.camera_operations())
      .field("file_operations", &self.file_operations())
      .field("folder_operations", &self.folder_operations())
      .field("device_type", &self.device_type())
      .field("usb_info", &self.usb_info())
      .finish()
  }
}

impl<'a> AbilitiesList<'a> {
  pub(crate) fn new(context: &Context) -> Result<Self> {
    let mut abilities_inner = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_abilities_list_new(&mut abilities_inner))?;
    try_gp_internal!(libgphoto2_sys::gp_abilities_list_load(abilities_inner, context.inner))?;

    Ok(Self { inner: abilities_inner, _phantom: PhantomData })
  }
}

impl Abilities {
  /// Camera ID
  pub fn id(&self) -> Cow<str> {
    chars_to_cow(self.inner.id.as_ptr())
  }

  /// Get the model of the camera
  pub fn model(&self) -> Cow<str> {
    chars_to_cow(self.inner.model.as_ptr())
  }

  /// Get the [driver status](CameraDriverStatus) of the device
  pub fn driver_status(&self) -> CameraDriverStatus {
    self.inner.status.into()
  }

  // TODO: Port speeds
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

  /// Get USB information
  pub fn usb_info(&self) -> UsbInfo {
    UsbInfo {
      vendor: self.inner.usb_vendor as usize,
      product: self.inner.usb_product as usize,
      class: self.inner.usb_class as usize,
      subclass: self.inner.usb_subclass as usize,
      protocol: self.inner.usb_protocol as usize,
    }
  }
}

macro_rules! impl_bitmask_check {
  ($(#[$meta:meta])*, $name:ident, $op:expr) => {
    $(#[$meta])*
    #[inline]
    pub fn $name(&self) -> bool {
      self.0 & $op as c_int != 0
    }
  };
}

impl CameraOperations {
  impl_bitmask_check!(
    /// The camera is able to capture images
    ,capture_image,
    libgphoto2_sys::CameraOperation::GP_OPERATION_CAPTURE_IMAGE
  );

  impl_bitmask_check!(
    /// The camera can capture videos
    ,capture_video,
    libgphoto2_sys::CameraOperation::GP_OPERATION_CAPTURE_VIDEO
  );

  impl_bitmask_check!(
    /// The camera can capture audio
    ,capture_audio,
    libgphoto2_sys::CameraOperation::GP_OPERATION_CAPTURE_AUDIO
  );

  impl_bitmask_check!(
    /// The camera can capture previews (small images that are not saved to the camera)
    ,capture_preview,
    libgphoto2_sys::CameraOperation::GP_OPERATION_CAPTURE_PREVIEW
  );

  impl_bitmask_check!(
    /// The camera can be configured
    ,config,
    libgphoto2_sys::CameraOperation::GP_OPERATION_CONFIG
  );

  impl_bitmask_check!(
    /// The camera can trigger captures
    ,trigger_capture,
    libgphoto2_sys::CameraOperation::GP_OPERATION_TRIGGER_CAPTURE
  );
}

impl FileOperations {
  impl_bitmask_check!(
    /// Files cam be deleted
    ,delete,
    libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_DELETE
  );

  impl_bitmask_check!(
    /// Previews of images
    ,preview,
    libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_PREVIEW
  );

  impl_bitmask_check!(
    /// Raw files
    ,raw,
    libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_RAW
  );

  impl_bitmask_check!(
    /// Get audio of file
    ,audio,
    libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_AUDIO
  );

  impl_bitmask_check!(
    /// Can get exif of files
    ,exif,
    libgphoto2_sys::CameraFileOperation::GP_FILE_OPERATION_EXIF
  );
}

impl FolderOperations {
  impl_bitmask_check!(
    /// Content of folder can be deleted
    ,delete_all,
    libgphoto2_sys::CameraFolderOperation::GP_FOLDER_OPERATION_DELETE_ALL
  );

  impl_bitmask_check!(
    /// Files can be uploaded to folder
    ,put_file,
    libgphoto2_sys::CameraFolderOperation::GP_FOLDER_OPERATION_PUT_FILE
  );

  impl_bitmask_check!(
    /// Directories can be created
    ,make_dir,
    libgphoto2_sys::CameraFolderOperation::GP_FOLDER_OPERATION_MAKE_DIR
  );

  impl_bitmask_check!(
    /// Directories can be removed
    ,remove_dir,
    libgphoto2_sys::CameraFolderOperation::GP_FOLDER_OPERATION_REMOVE_DIR
  );
}
