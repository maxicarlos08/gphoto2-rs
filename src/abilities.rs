//! Device abilities
//!
//! The device abilities describe the abilities of the driver used to connect to a device.

use crate::helper::{as_ref, bitflags, char_slice_to_cow};
use crate::{context::Context, try_gp_internal, Result};
use std::{borrow::Cow, fmt};

pub(crate) struct AbilitiesList {
  pub(crate) inner: *mut libgphoto2_sys::CameraAbilitiesList,
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
  pub(crate) inner: Box<libgphoto2_sys::CameraAbilities>,
}

/// Camera USB information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbInfo {
  /// Vendor ID
  pub vendor: u16,
  /// Product ID
  pub product: u16,
  /// Device class
  pub class: u8,
  /// Device subclass
  pub subclass: u8,
  /// Device protocol
  pub protocol: u8,
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

impl Drop for AbilitiesList {
  fn drop(&mut self) {
    try_gp_internal!(gp_abilities_list_free(self.inner).unwrap());
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

as_ref!(AbilitiesList -> libgphoto2_sys::CameraAbilitiesList, *self.inner);

as_ref!(Abilities -> libgphoto2_sys::CameraAbilities, self.inner);

impl AbilitiesList {
  /// Must be called from a [`Task`]
  pub(crate) fn new_inner(context: &Context) -> Result<Self> {
    try_gp_internal!(gp_abilities_list_new(&out abilities_inner)?);
    try_gp_internal!(gp_abilities_list_load(abilities_inner, context.inner)?);

    Ok(Self { inner: abilities_inner })
  }
}

impl Abilities {
  /// Camera ID
  pub fn id(&self) -> Cow<str> {
    char_slice_to_cow(&self.inner.id)
  }

  /// Get the model of the camera
  pub fn model(&self) -> Cow<str> {
    char_slice_to_cow(&self.inner.model)
  }

  /// Get the [driver status](CameraDriverStatus) of the device
  pub fn driver_status(&self) -> CameraDriverStatus {
    self.inner.status.into()
  }

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
    #[allow(clippy::as_conversions)]
    UsbInfo {
      vendor: self.inner.usb_vendor as u16,
      product: self.inner.usb_product as u16,
      class: self.inner.usb_class as u8,
      subclass: self.inner.usb_subclass as u8,
      protocol: self.inner.usb_protocol as u8,
    }
  }
}

bitflags!(
  /// Available operations on the camera
  CameraOperations = CameraOperation {
    /// The camera is able to capture images
    capture_image: GP_OPERATION_CAPTURE_IMAGE,

    /// The camera can capture videos
    capture_video: GP_OPERATION_CAPTURE_VIDEO,

    /// The camera can capture audio
    capture_audio: GP_OPERATION_CAPTURE_AUDIO,

    /// The camera can capture previews (small images that are not saved to the camera)
    capture_preview: GP_OPERATION_CAPTURE_PREVIEW,

    /// The camera can be configured
    configure: GP_OPERATION_CONFIG,

    /// The camera can trigger captures
    trigger_capture: GP_OPERATION_TRIGGER_CAPTURE,
  }
);

bitflags!(
  /// Available operations on files
  FileOperations = CameraFileOperation {
    /// Files cam be deleted
    delete: GP_FILE_OPERATION_DELETE,

    /// Previews of images
    preview: GP_FILE_OPERATION_PREVIEW,

    /// Raw files
    raw: GP_FILE_OPERATION_RAW,

    /// Get audio of file
    audio: GP_FILE_OPERATION_AUDIO,

    /// Can get exif of files
    exif: GP_FILE_OPERATION_EXIF,
  }
);

bitflags!(
  /// Available operations on folders
  FolderOperations = CameraFolderOperation {
    /// Content of folder can be deleted
    delete_all: GP_FOLDER_OPERATION_DELETE_ALL,

    /// Files can be uploaded to folder
    put_file: GP_FOLDER_OPERATION_PUT_FILE,

    /// Directories can be created
    make_dir: GP_FOLDER_OPERATION_MAKE_DIR,

    /// Directories can be removed
    remove_dir: GP_FOLDER_OPERATION_REMOVE_DIR,
  }
);
