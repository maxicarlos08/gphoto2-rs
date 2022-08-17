//! Camera filesystem and storages

use std::{borrow::Cow, os::raw::c_int};

use crate::helper::chars_to_cow;

macro_rules! storage_has_ability {
  ($inf:expr, $it:ident) => {
    $inf.fields as c_int & libgphoto2_sys::CameraStorageInfoFields::$it as c_int != 0
  };
}

/// Hardware storage type
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum StorageType {
  /// Unknown storage type
  Unknown,
  /// Fixed ROM storage
  FixedRom,
  /// Removable ROM storage
  RemovableRom,
  /// Fixed RAM
  FixedRam,
  /// Removable RAM storage (sd cards)
  RemovableRam,
}

/// Type of the filesystem hierarchy
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum FilesystemType {
  /// Unknown filesystem type
  Unknown,
  /// Flat filesystem (all in one directory)
  Flat,
  /// Tree hierarchy
  Tree,
  /// DCIM style filesystem
  Dcf,
}

/// Access types of storage
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum AccessType {
  /// Read/Write
  Rw,
  /// Read only
  Ro,
  /// Read only with delete
  RoDelete,
}

/// Information about a storage on the camera
pub struct StorageInfo {
  pub(crate) inner: libgphoto2_sys::CameraStorageInformation,
}

impl From<libgphoto2_sys::CameraStorageType> for StorageType {
  fn from(storage_type: libgphoto2_sys::CameraStorageType) -> Self {
    use libgphoto2_sys::CameraStorageType;

    match storage_type {
      CameraStorageType::GP_STORAGEINFO_ST_UNKNOWN => Self::Unknown,
      CameraStorageType::GP_STORAGEINFO_ST_FIXED_ROM => Self::FixedRom,
      CameraStorageType::GP_STORAGEINFO_ST_REMOVABLE_ROM => Self::RemovableRom,
      CameraStorageType::GP_STORAGEINFO_ST_FIXED_RAM => Self::FixedRam,
      CameraStorageType::GP_STORAGEINFO_ST_REMOVABLE_RAM => Self::RemovableRam,
    }
  }
}

impl From<libgphoto2_sys::CameraStorageFilesystemType> for FilesystemType {
  fn from(fs_type: libgphoto2_sys::CameraStorageFilesystemType) -> Self {
    use libgphoto2_sys::CameraStorageFilesystemType as GPFsType;

    match fs_type {
      GPFsType::GP_STORAGEINFO_FST_UNDEFINED => Self::Unknown,
      GPFsType::GP_STORAGEINFO_FST_GENERICFLAT => Self::Flat,
      GPFsType::GP_STORAGEINFO_FST_GENERICHIERARCHICAL => Self::Tree,
      GPFsType::GP_STORAGEINFO_FST_DCF => Self::Dcf,
    }
  }
}

impl From<libgphoto2_sys::CameraStorageAccessType> for AccessType {
  fn from(access_type: libgphoto2_sys::CameraStorageAccessType) -> Self {
    use libgphoto2_sys::CameraStorageAccessType as GPAccessType;

    match access_type {
      GPAccessType::GP_STORAGEINFO_AC_READWRITE => Self::Rw,
      GPAccessType::GP_STORAGEINFO_AC_READONLY => Self::Ro,
      GPAccessType::GP_STORAGEINFO_AC_READONLY_WITH_DELETE => Self::RoDelete,
    }
  }
}

impl StorageInfo {
  pub(crate) fn new(info: libgphoto2_sys::CameraStorageInformation) -> Self {
    Self { inner: info }
  }

  /// Base directory of the storage. If there is only 1 storage on the camera it will be "/"
  pub fn base_directory(&self) -> Option<Cow<str>> {
    if storage_has_ability!(self.inner, GP_STORAGEINFO_BASE) {
      Some(chars_to_cow(self.inner.basedir.as_ptr()))
    } else {
      None
    }
  }

  /// Label of the storage
  pub fn label(&self) -> Option<Cow<str>> {
    if storage_has_ability!(self.inner, GP_STORAGEINFO_LABEL) {
      Some(chars_to_cow(self.inner.label.as_ptr()))
    } else {
      None
    }
  }

  /// Description of the storage
  pub fn description(&self) -> Option<Cow<str>> {
    if storage_has_ability!(self.inner, GP_STORAGEINFO_DESCRIPTION) {
      Some(chars_to_cow(self.inner.description.as_ptr()))
    } else {
      None
    }
  }

  /// Type of the storage
  pub fn storage_type(&self) -> Option<StorageType> {
    if storage_has_ability!(self.inner, GP_STORAGEINFO_STORAGETYPE) {
      Some(self.inner.type_.into())
    } else {
      None
    }
  }

  /// Type of the filesystem on the storage
  pub fn filesystem_type(&self) -> Option<FilesystemType> {
    if storage_has_ability!(self.inner, GP_STORAGEINFO_FILESYSTEMTYPE) {
      Some(self.inner.fstype.into())
    } else {
      None
    }
  }

  /// Access permissions
  pub fn access_type(&self) -> Option<AccessType> {
    if storage_has_ability!(self.inner, GP_STORAGEINFO_ACCESS) {
      Some(self.inner.access.into())
    } else {
      None
    }
  }

  /// Total storage capacity in Kilobytes
  pub fn capacity(&self) -> Option<usize> {
    if storage_has_ability!(self.inner, GP_STORAGEINFO_MAXCAPACITY) {
      Some(self.inner.capacitykbytes as usize)
    } else {
      None
    }
  }

  /// Free storage in Kilobytes
  pub fn free(&self) -> Option<usize> {
    if storage_has_ability!(self.inner, GP_STORAGEINFO_FREESPACEKBYTES) {
      Some(self.inner.freekbytes as usize)
    } else {
      None
    }
  }

  /// Number of images that fit in free space (guessed by the camera)
  pub fn free_images(&self) -> Option<usize> {
    if storage_has_ability!(self.inner, GP_STORAGEINFO_FREESPACEIMAGES) {
      Some(self.inner.freeimages as usize)
    } else {
      None
    }
  }
}
