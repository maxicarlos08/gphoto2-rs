//! Camera filesystem and storages

use crate::{
  file::{CameraFile, FileType},
  helper::{chars_to_cow, to_c_string, uninit},
  list::CameraList,
  try_gp_internal, Camera, Result,
};
use std::{
  borrow::Cow,
  ffi, fmt,
  os::raw::{c_char, c_int},
};

macro_rules! storage_has_ability {
  ($inf:expr, $it:ident) => {
    $inf.fields as c_int & libgphoto2_sys::CameraStorageInfoFields::$it as c_int != 0
  };
}

macro_rules! file_info_get_field {
  ($info:expr, $if_field:ident, $field_name:ident) => {{
    if $info.fields as c_int & libgphoto2_sys::CameraFileInfoFields::$if_field as c_int != 0 {
      Some($info.$field_name)
    } else {
      None
    }
  }};
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

/// Status of [`CameraFile`]
pub enum FileStatus {
  /// The file was downloaded
  Downloaded,
  /// The file was not downloaded
  NotDownloaded,
}

/// Permissions of a [`CameraFile`]
pub struct FilePermissions(c_int);

/// Image thumbnail information
pub struct FileInfoPreview {
  /// Status of the preview file
  pub status: Option<FileStatus>,
  /// Size of the preview file
  pub size: Option<usize>,
  /// Mime type of the preview file
  pub mime_type: Option<String>,
  /// Image width,
  pub width: Option<usize>,
  /// Image height
  pub height: Option<usize>,
}

/// Info for image file
pub struct FileInfoFile {
  /// Status of the file
  pub status: Option<FileStatus>,
  /// File size
  pub size: Option<usize>,
  /// Mime type
  pub mime_type: Option<String>,
  /// Image width
  pub width: Option<usize>,
  /// Image height
  pub height: Option<usize>,
  /// Image permissions
  pub permissions: Option<FilePermissions>,
  /// File modification type
  pub mtime: Option<usize>,
}

/// Info for file audio data
pub struct FileInfoAudio {
  /// Status of the audio file
  pub status: Option<FileStatus>,
  /// Size of the audio file
  pub size: Option<usize>,
  /// Mime type of the audio
  pub mime_type: Option<String>,
}

/// File information for preview, normal file and audio
pub struct FileInfo {
  /// Info for file preview
  pub preview: FileInfoPreview,
  /// Info for normal file
  pub file: FileInfoFile,
  /// Info for file audio
  pub audio: FileInfoAudio,
}

/// File system actions for a camera
pub struct CameraFS<'a> {
  pub(crate) camera: &'a Camera<'a>,
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

impl From<libgphoto2_sys::CameraFileStatus> for FileStatus {
  fn from(status: libgphoto2_sys::CameraFileStatus) -> Self {
    use libgphoto2_sys::CameraFileStatus;

    match status {
      CameraFileStatus::GP_FILE_STATUS_DOWNLOADED => Self::Downloaded,
      CameraFileStatus::GP_FILE_STATUS_NOT_DOWNLOADED => Self::NotDownloaded,
    }
  }
}

impl From<libgphoto2_sys::CameraFileInfoPreview> for FileInfoPreview {
  fn from(preview_info: libgphoto2_sys::CameraFileInfoPreview) -> Self {
    Self {
      status: file_info_get_field!(preview_info, GP_FILE_INFO_STATUS, status).map(Into::into),
      size: file_info_get_field!(preview_info, GP_FILE_INFO_SIZE, size).map(|size| size as usize),
      mime_type: file_info_get_field!(preview_info, GP_FILE_INFO_TYPE, type_)
        .map(|mime| chars_to_cow(mime.as_ptr()).to_string()),
      width: file_info_get_field!(preview_info, GP_FILE_INFO_WIDTH, width)
        .map(|width| width as usize),
      height: file_info_get_field!(preview_info, GP_FILE_INFO_HEIGHT, height)
        .map(|height| height as usize),
    }
  }
}

impl From<libgphoto2_sys::CameraFilePermissions> for FilePermissions {
  fn from(permissions: libgphoto2_sys::CameraFilePermissions) -> Self {
    Self(permissions as c_int)
  }
}

impl From<libgphoto2_sys::CameraFileInfoFile> for FileInfoFile {
  fn from(file_info: libgphoto2_sys::CameraFileInfoFile) -> Self {
    Self {
      status: file_info_get_field!(file_info, GP_FILE_INFO_STATUS, status).map(Into::into),
      size: file_info_get_field!(file_info, GP_FILE_INFO_SIZE, size).map(|size| size as usize),
      mime_type: file_info_get_field!(file_info, GP_FILE_INFO_TYPE, type_)
        .map(|mime| chars_to_cow(mime.as_ptr()).to_string()),
      width: file_info_get_field!(file_info, GP_FILE_INFO_WIDTH, width).map(|width| width as usize),
      height: file_info_get_field!(file_info, GP_FILE_INFO_HEIGHT, height)
        .map(|height| height as usize),
      permissions: file_info_get_field!(file_info, GP_FILE_INFO_PERMISSIONS, permissions)
        .map(Into::into),
      mtime: file_info_get_field!(file_info, GP_FILE_INFO_MTIME, mtime).map(|mtime| mtime as usize),
    }
  }
}

impl From<libgphoto2_sys::CameraFileInfoAudio> for FileInfoAudio {
  fn from(audio_info: libgphoto2_sys::CameraFileInfoAudio) -> Self {
    Self {
      status: file_info_get_field!(audio_info, GP_FILE_INFO_STATUS, status).map(Into::into),
      size: file_info_get_field!(audio_info, GP_FILE_INFO_SIZE, size).map(|size| size as usize),
      mime_type: file_info_get_field!(audio_info, GP_FILE_INFO_TYPE, type_)
        .map(|mime| chars_to_cow(mime.as_ptr()).to_string()),
    }
  }
}

impl From<libgphoto2_sys::CameraFileInfo> for FileInfo {
  fn from(info: libgphoto2_sys::CameraFileInfo) -> Self {
    Self { preview: info.preview.into(), file: info.file.into(), audio: info.audio.into() }
  }
}

impl fmt::Debug for StorageInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("StorageInfo")
      .field("label", &self.label())
      .field("base_directory", &self.base_directory())
      .field("description", &self.description())
      .field("storage_type", &self.storage_type())
      .field("filesystem_type", &self.filesystem_type())
      .field("access_type", &self.access_type())
      .field("capacity", &self.capacity())
      .field("free", &self.free())
      .field("free_images", &self.free_images())
      .finish()
  }
}

impl FilePermissions {
  /// File can be read
  pub fn read(&self) -> bool {
    self.0 & libgphoto2_sys::CameraFilePermissions::GP_FILE_PERM_READ as c_int != 0
  }

  /// File can be deleted
  pub fn delete(&self) -> bool {
    self.0 & libgphoto2_sys::CameraFilePermissions::GP_FILE_PERM_DELETE as c_int != 0
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

impl<'a> CameraFS<'a> {
  pub(crate) fn new(camera: &'a Camera) -> Self {
    Self { camera }
  }

  /// Delete a file
  pub fn delete_file(&self, folder: &str, file: &str) -> Result<()> {
    to_c_string!(folder, folder);
    to_c_string!(file, file);

    try_gp_internal!(libgphoto2_sys::gp_camera_file_delete(
      self.camera.camera,
      folder.as_ptr() as *const c_char,
      file.as_ptr() as *const c_char,
      self.camera.context.inner
    ))?;
    Ok(())
  }

  /// Get information of a file
  pub fn info(&self, folder: &str, file: &str) -> Result<FileInfo> {
    let mut file_info = unsafe { uninit() };

    to_c_string!(folder);
    to_c_string!(file);

    try_gp_internal!(libgphoto2_sys::gp_camera_file_get_info(
      self.camera.camera,
      folder.as_ptr() as *const c_char,
      file.as_ptr() as *const c_char,
      &mut file_info,
      self.camera.context.inner
    ))?;

    Ok(file_info.into())
  }

  /// Upload a file to the camera
  pub fn upload_file(&self, folder: &str, filename: &str, file: CameraFile) -> Result<()> {
    to_c_string!(folder);
    to_c_string!(filename);

    try_gp_internal!(libgphoto2_sys::gp_camera_folder_put_file(
      self.camera.camera,
      folder.as_ptr() as *const c_char,
      filename.as_ptr() as *const c_char,
      FileType::Normal.into(),
      file.inner,
      self.camera.context.inner
    ))?;

    Ok(())
  }

  /// Delete all files in a folder
  pub fn folder_delete_all(&self, folder: &str) -> Result<()> {
    to_c_string!(folder);

    try_gp_internal!(libgphoto2_sys::gp_camera_folder_delete_all(
      self.camera.camera,
      folder.as_ptr() as *const c_char,
      self.camera.context.inner
    ))?;
    Ok(())
  }

  /// List files in a folder
  pub fn ls_files(&self, folder: &str) -> Result<Vec<String>> {
    let file_list = CameraList::new()?;

    to_c_string!(folder);

    try_gp_internal!(libgphoto2_sys::gp_camera_folder_list_files(
      self.camera.camera,
      folder.as_ptr() as *const c_char,
      file_list.inner,
      self.camera.context.inner
    ))?;

    Ok(file_list.to_vec()?.into_iter().map(|(name, _)| name.to_string()).collect())
  }

  /// List folders in a folder
  pub fn ls_folder(&self, folder: &str) -> Result<Vec<String>> {
    let folder_list = CameraList::new()?;

    to_c_string!(folder);

    try_gp_internal!(libgphoto2_sys::gp_camera_folder_list_files(
      self.camera.camera,
      folder.as_ptr() as *const c_char,
      folder_list.inner,
      self.camera.context.inner
    ))?;

    Ok(folder_list.to_vec()?.into_iter().map(|(name, _)| name.to_string()).collect())
  }

  /// Creates a new folder
  pub fn mkdir(&self, parent_folder: &str, new_folder: &str) -> Result<()> {
    to_c_string!(parent_folder);
    to_c_string!(new_folder);

    try_gp_internal!(libgphoto2_sys::gp_camera_folder_make_dir(
      self.camera.camera,
      parent_folder.as_ptr() as *const c_char,
      new_folder.as_ptr() as *const c_char,
      self.camera.context.inner
    ))?;

    Ok(())
  }

  /// Removes a folder
  pub fn rmdir(&self, parent: &str, to_remove: &str) -> Result<()> {
    to_c_string!(parent);
    to_c_string!(to_remove);

    try_gp_internal!(libgphoto2_sys::gp_camera_folder_remove_dir(
      self.camera.camera,
      parent.as_ptr() as *const c_char,
      to_remove.as_ptr() as *const c_char,
      self.camera.context.inner
    ))?;

    Ok(())
  }
}
