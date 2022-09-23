use std::mem::MaybeUninit;
use std::{ffi, os::raw::c_char};

pub fn char_slice_to_str(chars: &[c_char]) -> &'_ str {
  unsafe { ffi::CStr::from_ptr(chars.as_ptr()).to_str().expect("Got invalid utf from libgphoto2") }
}

pub fn chars_to_string(chars: *const c_char) -> String {
  unsafe { String::from_utf8_lossy(ffi::CStr::from_ptr(chars).to_bytes()) }.into_owned()
}

pub struct UninitBox<T> {
  inner: Box<MaybeUninit<T>>,
}

impl<T> UninitBox<T> {
  pub fn uninit() -> Self {
    Self { inner: Box::new(MaybeUninit::uninit()) }
  }

  pub fn as_mut_ptr(&mut self) -> *mut T {
    self.inner.as_mut_ptr().cast()
  }

  pub unsafe fn assume_init(self) -> Box<T> {
    Box::from_raw(Box::into_raw(self.inner).cast())
  }
}

macro_rules! to_c_string {
  ($v:expr) => {
    ffi::CString::new($v)?.as_ptr().cast::<std::os::raw::c_char>()
  };
}

macro_rules! as_ref {
  ($from:ident -> $to:ty, $self:ident $($rest:tt)*) => {
    as_ref!(@ $from -> $to, , $self, $self $($rest)*);
  };

  ($from:ident -> $to:ty, * $self:ident $($rest:tt)*) => {
    as_ref!(@ $from -> $to, unsafe, $self, *$self $($rest)*);
  };

  (@ $from:ident -> $to:ty, $($unsafe:ident)?, $self:ident, $value:expr) => {
    impl AsRef<$to> for $from {
      fn as_ref(&$self) -> &$to {
        $($unsafe)? { & $value }
      }
    }

    impl AsMut<$to> for $from {
      fn as_mut(&mut $self) -> &mut $to {
        $($unsafe)? { &mut $value }
      }
    }
  };
}

macro_rules! bitflags {
  ($(# $attr:tt)* $name:ident = $target:ident { $($(# $field_attr:tt)* $field:ident: $value:ident,)* }) => {
    $(# $attr)*
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct $name(libgphoto2_sys::$target);

    impl From<libgphoto2_sys::$target> for $name {
      fn from(flags: libgphoto2_sys::$target) -> Self {
        Self(flags)
      }
    }

    impl $name {
      $(
        $(# $field_attr)*
        #[inline]
        pub fn $field(&self) -> bool {
          (self.0 & libgphoto2_sys::$target::$value).0 != 0
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

pub(crate) use {as_ref, bitflags, to_c_string};
