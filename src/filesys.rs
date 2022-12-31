//! Camera filesystem and storages

use crate::{
  file::{CameraFile, FileType},
  helper::{bitflags, char_slice_to_cow, to_c_string, UninitBox},
  list::{CameraList, FileListIter},
  task::Task,
  try_gp_internal, Camera, Result,
};
use libgphoto2_sys::time_t;
use std::{borrow::Cow, ffi, fmt, path::Path};

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
        let ptr: *const _ = ptr;
        // Safe because of repr(transparent).
        unsafe { &*ptr.cast::<Self>() }
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

    impl fmt::Debug for $name {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    size: u64 = GP_FILE_INFO_SIZE, info.size;
    /// Mime type of the preview file
    mime_type: Cow<str> = GP_FILE_INFO_TYPE, char_slice_to_cow(&info.type_);
    /// Image width,
    width: u32 = GP_FILE_INFO_WIDTH, info.width;
    /// Image height
    height: u32 = GP_FILE_INFO_HEIGHT, info.height;
  }
);

storage_info!(
  /// Info for image file
  FileInfoFile: CameraFileInfoFields, |info: CameraFileInfoFile| {
    /// Status of the file
    status: FileStatus = GP_FILE_INFO_STATUS, info.status.into();
    /// File size
    size: u64 = GP_FILE_INFO_SIZE, info.size;
    /// Mime type
    mime_type: Cow<str> = GP_FILE_INFO_TYPE, char_slice_to_cow(&info.type_);
    /// Image width
    width: u32 = GP_FILE_INFO_WIDTH, info.width;
    /// Image height
    height: u32 = GP_FILE_INFO_HEIGHT, info.height;
    /// Image permissions
    permissions: FilePermissions = GP_FILE_INFO_PERMISSIONS, info.permissions.into();
    /// File modification time
    mtime: time_t = GP_FILE_INFO_MTIME, info.mtime;
  }
);

storage_info!(
  /// Info for file audio data
  FileInfoAudio: CameraFileInfoFields, |info: CameraFileInfoAudio| {
    /// Status of the audio file
    status: FileStatus = GP_FILE_INFO_STATUS, info.status.into();
    /// Size of the audio file
    size: u64 = GP_FILE_INFO_SIZE, info.size;
    /// Mime type of the audio
    mime_type: Cow<str> = GP_FILE_INFO_TYPE, char_slice_to_cow(&info.type_);
  }
);

/// File information for preview, normal file and audio
pub struct FileInfo {
  // It's fairly large, so we want to keep it on the heap.
  pub(crate) inner: Box<libgphoto2_sys::CameraFileInfo>,
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

impl fmt::Debug for FileInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("FileInfo")
      .field("preview", &self.preview())
      .field("file", &self.file())
      .field("audio", &self.audio())
      .finish()
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
    capacity_kb: u64 = GP_STORAGEINFO_MAXCAPACITY, info.capacitykbytes * 1024;
    /// Free storage in Kilobytes
    free_kb: u64 = GP_STORAGEINFO_FREESPACEKBYTES, info.freekbytes * 1024;
    /// Number of images that fit in free space (guessed by the camera)
    free_images: u64 = GP_STORAGEINFO_FREESPACEIMAGES, info.freeimages;
  }
);

impl<'a> CameraFS<'a> {
  pub(crate) fn new(camera: &'a Camera) -> Self {
    Self { camera }
  }

  /// Delete a file
  pub fn delete_file(&self, folder: &str, file: &str) -> Task<Result<()>> {
    let camera = self.camera.camera;
    let context = self.camera.context.inner;
    let (folder, file) = (folder.to_owned(), file.to_owned());

    unsafe {
      Task::new(move || {
        try_gp_internal!(gp_camera_file_delete(
          *camera,
          to_c_string!(folder),
          to_c_string!(file),
          *context
        )?);
        Ok(())
      })
    }
    .context(context)
  }

  /// Get information of a file
  pub fn file_info(&self, folder: &str, file: &str) -> Task<Result<FileInfo>> {
    let camera = self.camera.camera;
    let context = self.camera.context.inner;
    let (folder, file) = (folder.to_owned(), file.to_owned());

    unsafe {
      Task::new(move || {
        let mut inner = UninitBox::uninit();

        try_gp_internal!(gp_camera_file_get_info(
          *camera,
          to_c_string!(folder),
          to_c_string!(file),
          inner.as_mut_ptr(),
          *context
        )?);

        Ok(FileInfo { inner: inner.assume_init() })
      })
    }
    .context(context)
  }

  /// Downloads a file from the camera
  pub fn download_to(&self, folder: &str, file: &str, path: &Path) -> Task<Result<CameraFile>> {
    self.to_camera_file(folder, file, FileType::Normal, Some(path))
  }

  /// Downloads a camera file to memory
  pub fn download(&self, folder: &str, file: &str) -> Task<Result<CameraFile>> {
    self.to_camera_file(folder, file, FileType::Normal, None)
  }

  /// Downloads a preview into memory
  pub fn download_preview(&self,folder: &str, file: &str) -> Task<Result<CameraFile>> {
    self.to_camera_file(folder, file, FileType::Preview, None)
  }

  /// Upload a file to the camera
  #[allow(clippy::boxed_local)]
  pub fn upload_file(&self, folder: &str, filename: &str, data: Box<[u8]>) -> Task<Result<()>> {
    let camera = self.camera.camera;
    let context = self.camera.context.inner;

    let (folder, filename) = (folder.to_owned(), filename.to_owned());

    unsafe {
      Task::new(move || {
        try_gp_internal!(gp_file_new(&out file)?);
        try_gp_internal!(gp_file_append(file, data.as_ptr().cast(), data.len().try_into()?)?);
        try_gp_internal!(gp_camera_folder_put_file(
          *camera,
          to_c_string!(folder),
          to_c_string!(filename),
          FileType::Normal.into(),
          file,
          *context
        )?);

        Ok(())
      })
    }
    .context(context)
  }

  /// Delete all files in a folder
  pub fn delete_all_in_folder(&self, folder: &str) -> Task<Result<()>> {
    let camera = self.camera.camera;
    let context = self.camera.context.inner;
    let folder = folder.to_owned();

    unsafe {
      Task::new(move || {
        try_gp_internal!(gp_camera_folder_delete_all(*camera, to_c_string!(folder), *context)?);
        Ok(())
      })
    }
    .context(context)
  }

  /// List files in a folder
  pub fn list_files(&self, folder: &str) -> Task<Result<FileListIter>> {
    let camera = self.camera.camera;
    let context = self.camera.context.inner;

    let folder = folder.to_owned();

    unsafe {
      Task::new(move || {
        let file_list = CameraList::new()?;

        try_gp_internal!(gp_camera_folder_list_files(
          *camera,
          to_c_string!(folder),
          *file_list.inner,
          *context
        )?);

        Ok(FileListIter::new(file_list))
      })
    }
    .context(context)
  }

  /// List folders in a folder
  pub fn list_folders(&self, folder: &str) -> Task<Result<FileListIter>> {
    let camera = self.camera.camera;
    let context = self.camera.context.inner;

    let folder = folder.to_owned();

    unsafe {
      Task::new(move || {
        let folder_list = CameraList::new()?;

        try_gp_internal!(gp_camera_folder_list_folders(
          *camera,
          to_c_string!(folder),
          *folder_list.inner,
          *context
        )?);

        Ok(FileListIter::new(folder_list))
      })
    }
    .context(context)
  }

  /// Creates a new folder
  pub fn create_directory(&self, parent_folder: &str, new_folder: &str) -> Task<Result<()>> {
    let (parent_folder, new_folder) = (parent_folder.to_owned(), new_folder.to_owned());
    let camera = self.camera.camera;
    let context = self.camera.context.inner;

    unsafe {
      Task::new(move || {
        try_gp_internal!(gp_camera_folder_make_dir(
          *camera,
          to_c_string!(parent_folder),
          to_c_string!(new_folder),
          *context
        )?);

        Ok(())
      })
    }
    .context(context)
  }

  /// Removes a folder
  pub fn remove_directory(&self, parent: &str, to_remove: &str) -> Task<Result<()>> {
    let (parent, to_remove) = (parent.to_owned(), to_remove.to_owned());
    let camera = self.camera.camera;
    let context = self.camera.context.inner;

    unsafe {
      Task::new(move || {
        try_gp_internal!(gp_camera_folder_remove_dir(
          *camera,
          to_c_string!(parent),
          to_c_string!(to_remove),
          *context
        )?);

        Ok(())
      })
    }
    .context(context)
  }
}

/// Private implementations
impl CameraFS<'_> {
  fn to_camera_file(
    &self,
    folder: &str,
    file: &str,
    type_: FileType,
    path: Option<&Path>,
  ) -> Task<Result<CameraFile>> {
    let (folder, file, path) = (folder.to_owned(), file.to_owned(), path.map(ToOwned::to_owned));
    let camera = self.camera.camera;
    let context = self.camera.context.inner;

    unsafe {
      Task::new(move || {
        let camera_file = match path {
          Some(dest_path) => CameraFile::new_file(&dest_path)?,
          None => CameraFile::new()?,
        };

        try_gp_internal!(gp_camera_file_get(
          *camera,
          to_c_string!(folder),
          to_c_string!(file),
          type_.into(),
          *camera_file.inner,
          *context
        )?);

        Ok(camera_file)
      })
    }
    .context(context)
  }
}
