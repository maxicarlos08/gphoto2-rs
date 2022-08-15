//! Camera port information

use crate::{helper::chars_to_cow, try_gp_internal, Result};
use std::{borrow::Cow, mem::MaybeUninit};

/// Type of the port
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortType {
  /// Serial port
  Serial,
  /// USB port
  Usb,
  /// Disk
  Disk,

  /// PTP/IP
  PTPIp,

  /// IP
  Ip,

  /// USB Disk direct
  UsbDiskDirect,

  /// USB SCSI
  UsbScsi,
}

/// Information about the port
pub struct PortInfo {
  pub(crate) inner: libgphoto2_sys::GPPortInfo,
}

pub(crate) struct PortInfoList {
  pub(crate) inner: *mut libgphoto2_sys::GPPortInfoList,
}

impl Drop for PortInfoList {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_port_info_list_free(self.inner);
    }
  }
}

impl From<libgphoto2_sys::GPPortInfo> for PortInfo {
  fn from(inner: libgphoto2_sys::GPPortInfo) -> Self {
    Self { inner }
  }
}

impl PortType {
  fn new(port_type: libgphoto2_sys::GPPortType) -> Option<Self> {
    use libgphoto2_sys::GPPortType;

    match port_type {
      GPPortType::GP_PORT_NONE => None,
      GPPortType::GP_PORT_SERIAL => Some(Self::Serial),
      GPPortType::GP_PORT_USB => Some(Self::Usb),
      GPPortType::GP_PORT_DISK => Some(Self::Disk),
      GPPortType::GP_PORT_PTPIP => Some(Self::PTPIp),
      GPPortType::GP_PORT_IP => Some(Self::Ip),
      GPPortType::GP_PORT_USB_DISK_DIRECT => Some(Self::UsbDiskDirect),
      GPPortType::GP_PORT_USB_SCSI => Some(Self::UsbScsi),
    }
  }
}

impl PortInfo {
  /// Name of the port
  pub fn name(&self) -> Result<Cow<str>> {
    let mut name = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_port_info_get_name(self.inner, &mut name))?;

    Ok(chars_to_cow(name))
  }

  /// Path of the port
  pub fn path(&self) -> Result<Cow<str>> {
    let mut path = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_port_info_get_path(self.inner, &mut path))?;

    Ok(chars_to_cow(path))
  }

  /// [Port type](PortType)
  pub fn port_type(&self) -> Result<Option<PortType>> {
    let mut port_type = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_port_info_get_type(self.inner, &mut port_type))?;

    Ok(PortType::new(port_type))
  }
}

impl PortInfoList {
  pub(crate) fn new() -> Result<Self> {
    let mut port_info_list = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_port_info_list_new(&mut port_info_list))?;
    try_gp_internal!(libgphoto2_sys::gp_port_info_list_load(port_info_list))?;

    Ok(Self { inner: port_info_list })
  }

  pub(crate) fn get_port_info(&self, p: i32) -> Result<PortInfo> {
    let mut port_info = unsafe { MaybeUninit::zeroed().assume_init() };

    try_gp_internal!(libgphoto2_sys::gp_port_info_list_get_info(self.inner, p, &mut port_info))?;

    todo!()
  }
}
