//! Camera filesystem and storages

use crate::{
  file::{CameraFile, FileType},
  helper::{bitflags, char_slice_to_cow, to_c_string, UninitBox},
  list::CameraList,
  try_gp_internal, Camera, Result,
};
use std::{borrow::Cow, ffi};

macro_rules! storage_info {
  ($(# $attr:tt)* $name:ident: $bitflag_ty:ident, |$inner:ident: $inner_ty:ident| { $($(# $field_attr:tt)* $field:ident: $ty:ty = $bitflag:ident, $expr:expr;)* }) => {
    $(# $attr)*
    #[repr(transparent)]
    #[derive(Clone)]
    pub struct $name {
      inner: libgphoto2_sys::$inner_ty,
    }

    impl $name {
      #[allow(dead_code)]
      pub(crate) fn from_inner_ref(ptr: &libgphoto2_sys::$inner_ty) -> &Self {
        // Safe because of repr(transparent).
        unsafe { &*(ptr as *const _ as *const Self) }
      }

      $(
        $(# $field_attr)*
        pub fn $field(&self) -> Option<$ty> {
          let $inner = &self.inner;
          if ($inner.fields & libgphoto2_sys::$bitflag_ty::$bitflag).0 != 0 {
            Some($expr)
          } else {
            None
          }
        }
      )*
    }

    impl std::fmt::Debug for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!($name))
          $(
            .field(stringify!($field), &self.$field())
          )*
          .finish()
      }
    }
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

bitflags!(
  /// Status of [`CameraFile`].
  FileStatus = CameraFileStatus {
    /// This file has been downloaded.
    downloaded: GP_FILE_STATUS_DOWNLOADED,
  }
);

bitflags!(
  /// Permissions of a [`CameraFile`]
  FilePermissions = CameraFilePermissions {
    /// File can be read
    read: GP_FILE_PERM_READ,

    /// File can be deleted
    delete: GP_FILE_PERM_DELETE,
  }
);

storage_info!(
  /// Image thumbnail information
  FileInfoPreview: CameraFileInfoFields, |info: CameraFileInfoPreview| {
    /// Status of the preview file
    status: FileStatus = GP_FILE_INFO_STATUS, info.status.into();
    /// Size of the preview file
    size: usize = GP_FILE_INFO_SIZE, info.size as usize;
    /// Mime type of the preview file
    mime_type: Cow<str> = GP_FILE_INFO_TYPE, char_slice_to_cow(&info.type_);
    /// Image width,
    width: usize = GP_FILE_INFO_WIDTH, info.width as usize;
    /// Image height
    height: usize = GP_FILE_INFO_HEIGHT, info.height as usize;
  }
);

storage_info!(
  /// Info for image file
  FileInfoFile: CameraFileInfoFields, |info: CameraFileInfoFile| {
    /// Status of the file
    status: FileStatus = GP_FILE_INFO_STATUS, info.status.into();
    /// File size
    size: usize = GP_FILE_INFO_SIZE, info.size as usize;
    /// Mime type
    mime_type: Cow<str> = GP_FILE_INFO_TYPE, char_slice_to_cow(&info.type_);
    /// Image width
    width: usize = GP_FILE_INFO_WIDTH, info.width as usize;
    /// Image height
    height: usize = GP_FILE_INFO_HEIGHT, info.height as usize;
    /// Image permissions
    permissions: FilePermissions = GP_FILE_INFO_PERMISSIONS, info.permissions.into();
    /// File modification type
    mtime: usize = GP_FILE_INFO_MTIME, info.mtime as usize;
  }
);

storage_info!(
  /// Info for file audio data
  FileInfoAudio: CameraFileInfoFields, |info: CameraFileInfoAudio| {
    /// Status of the audio file
    status: FileStatus = GP_FILE_INFO_STATUS, info.status.into();
    /// Size of the audio file
    size: usize = GP_FILE_INFO_SIZE, info.size as usize;
    /// Mime type of the audio
    mime_type: Cow<str> = GP_FILE_INFO_TYPE, char_slice_to_cow(&info.type_);
  }
);

/// File information for preview, normal file and audio
pub struct FileInfo {
  // It's fairly large, so we want to keep it on the heap.
  inner: Box<libgphoto2_sys::CameraFileInfo>,
}

impl FileInfo {
  /// Info for file preview
  pub fn preview(&self) -> &FileInfoPreview {
    FileInfoPreview::from_inner_ref(&self.inner.preview)
  }

  /// Info for normal file
  pub fn file(&self) -> &FileInfoFile {
    FileInfoFile::from_inner_ref(&self.inner.file)
  }

  /// Info for file audio
  pub fn audio(&self) -> &FileInfoAudio {
    FileInfoAudio::from_inner_ref(&self.inner.audio)
  }
}

/// File system actions for a camera
pub struct CameraFS<'a> {
  pub(crate) camera: &'a Camera,
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

storage_info!(
  /// Information about a storage on the camera
  StorageInfo: CameraStorageInfoFields, |info: CameraStorageInformation| {
    /// Label of the storage
    label: Cow<str> = GP_STORAGEINFO_LABEL, char_slice_to_cow(&info.label);
    /// Base directory of the storage. If there is only 1 storage on the camera it will be "/"
    base_directory: Cow<str> = GP_STORAGEINFO_BASE, char_slice_to_cow(&info.basedir);
    /// Description of the storage
    description: Cow<str> = GP_STORAGEINFO_DESCRIPTION, char_slice_to_cow(&info.description);
    /// Type of the storage
    storage_type: StorageType = GP_STORAGEINFO_STORAGETYPE, info.type_.into();
    /// Type of the filesystem on the storage
    filesystem_type: FilesystemType = GP_STORAGEINFO_FILESYSTEMTYPE, info.fstype.into();
    /// Access permissions
    access_type: AccessType = GP_STORAGEINFO_ACCESS, info.access.into();
    /// Total storage capacity in Kilobytes
    capacity_kb: usize = GP_STORAGEINFO_MAXCAPACITY, info.capacitykbytes as usize * 1024;
    /// Free storage in Kilobytes
    free_kb: usize = GP_STORAGEINFO_FREESPACEKBYTES, info.freekbytes as usize * 1024;
    /// Number of images that fit in free space (guessed by the camera)
    free_images: usize = GP_STORAGEINFO_FREESPACEIMAGES, info.freeimages as usize;
  }
);

impl<'a> CameraFS<'a> {
  pub(crate) fn new(camera: &'a Camera) -> Self {
    Self { camera }
  }

  /// Delete a file
  pub fn delete_file(&self, folder: &str, file: &str) -> Result<()> {
    try_gp_internal!(gp_camera_file_delete(
      self.camera.camera,
      to_c_string!(folder),
      to_c_string!(file),
      self.camera.context
    ));
    Ok(())
  }

  /// Get information of a file
  pub fn info(&self, folder: &str, file: &str) -> Result<FileInfo> {
    let mut inner = UninitBox::uninit();

    try_gp_internal!(gp_camera_file_get_info(
      self.camera.camera,
      to_c_string!(folder),
      to_c_string!(file),
      inner.as_mut_ptr(),
      self.camera.context
    ));

    Ok(FileInfo { inner: unsafe { inner.assume_init() } })
  }

  /// Upload a file to the camera
  pub fn upload_file(&self, folder: &str, filename: &str, file: CameraFile) -> Result<()> {
    try_gp_internal!(gp_camera_folder_put_file(
      self.camera.camera,
      to_c_string!(folder),
      to_c_string!(filename),
      FileType::Normal.into(),
      file.inner,
      self.camera.context
    ));

    Ok(())
  }

  /// Delete all files in a folder
  pub fn folder_delete_all(&self, folder: &str) -> Result<()> {
    try_gp_internal!(gp_camera_folder_delete_all(
      self.camera.camera,
      to_c_string!(folder),
      self.camera.context
    ));
    Ok(())
  }

  /// List files in a folder
  pub fn ls_files(&self, folder: &str) -> Result<Vec<String>> {
    let file_list = CameraList::new()?;

    try_gp_internal!(gp_camera_folder_list_files(
      self.camera.camera,
      to_c_string!(folder),
      file_list.inner,
      self.camera.context
    ));

    Ok(file_list.to_vec()?.into_iter().map(|(name, _)| name).collect())
  }

  /// List folders in a folder
  pub fn ls_folders(&self, folder: &str) -> Result<Vec<String>> {
    let folder_list = CameraList::new()?;

    try_gp_internal!(gp_camera_folder_list_folders(
      self.camera.camera,
      to_c_string!(folder),
      folder_list.inner,
      self.camera.context
    ));

    Ok(folder_list.to_vec()?.into_iter().map(|(name, _)| name).collect())
  }

  /// Creates a new folder
  pub fn mkdir(&self, parent_folder: &str, new_folder: &str) -> Result<()> {
    try_gp_internal!(gp_camera_folder_make_dir(
      self.camera.camera,
      to_c_string!(parent_folder),
      to_c_string!(new_folder),
      self.camera.context
    ));

    Ok(())
  }

  /// Removes a folder
  pub fn rmdir(&self, parent: &str, to_remove: &str) -> Result<()> {
    try_gp_internal!(gp_camera_folder_remove_dir(
      self.camera.camera,
      to_c_string!(parent),
      to_c_string!(to_remove),
      self.camera.context
    ));

    Ok(())
  }
}
