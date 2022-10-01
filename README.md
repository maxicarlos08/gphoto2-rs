# GPhoto2-rs

Rust bindings to libgphoto2

### What about [gphoto-rs](https://crates.io/crates/gphoto)?

I know about the other crate ([gphoto](https://crates.io/crates/gphoto) and [gphoto2-sys](https://crates.io/crates/gphoto2-sys) which was created by [@dcuddeback](https://github.com/dcuddeback/), but it is missing a lot of features which make the crate unusable for me, most notably the ability to change camera settings and in memory file download.  
The author hasn't been active since 2017 regardless of numerous pull- and feature requests, so I made a new project with a more up to date rust code and all the features from libgphoto2.

## Features

- [x] Camera
  - [x] Capture images
  - [x] Capture preview images
  - [x] Download images
  - [x] Get port information
  - [x] Get abilities (model, driver stability, permissions, ...)
  - [x] Read configuration
  - [x] Set configuration
  - [x] Interact with filesystem on camera
  - [x] Camera events
  - [x] Usb port information
- [x] Context
  - [x] Autodetect camera
  - [x] Get list of available cameras
  - [x] Get camera by model and port

## Gettings started

### Installation

Run `cargo add gphoto2` to add gphoto2 to your project or add this to your `Cargo.toml`:

```toml
[dependencies]
gphoto2 = "1"
```

#### Install libgphoto2

The `libgphoto2` library must be installed on your system to use this library.

To install libgphoto2 on Debian based systems run:

```sh
sudo apt install libgphoto2-dev
```

On Arch systems run:

```sh
sudo pacman -S libgphoto2
```

On MacOS systems with Homebrew run:

```sh
homebrew install libgphoto2
```

##### Windows

There is no official way to install libgphoto2 on windows, but you can install it with [MSYS2](https://www.msys2.org/) (link to the package: [mingw-w64-libgphoto2](https://packages.msys2.org/package/mingw-w64-x86_64-libgphoto2)).

## Basic Usage

This example takes a picture and saves it to disk

```rust no_run
use gphoto2::{Context, Result};
use std::path::Path;

fn main() -> Result<()> {
  // Create a new context and detect the first camera from it
  let camera = Context::new()?.autodetect_camera().expect("Failed to autodetect camera");
  let camera_fs = camera.fs();


  // And take pictures
  let file_path = camera.capture_image().expect("Could not capture image");
  camera_fs.download_to(&file_path.name(), &file_path.folder(), Path::new(&file_path.name().to_string()))?;

  // For more advanced examples take a look at the examples/ folder

  Ok(())
}
```

You can find more examples [here](https://github.com/maxicarlos08/gphoto2-rs/tree/master/examples)

## Logging

To make your debugging life a bit easier, this crate hooks up the libgphoto2 log functions to the [`log`](https://docs.rs/log) crate.

To show the logs use a logging implementation like [`env_logger`](https://crates.io/crates/env_logger).

### Additional logs

By default we use `gp_context_set_log_func` in a context to get the logs, but there is also `gp_log_add_func` which provides a lot more information and can be a lot more useful when debugging.

The reason this crate doesn't use `gp_log_add_func` by default is because it is disabled in most Linux distributions and Windows. You will have to check if your installed version does not disable this feature or build libgphoto2 yourself **without** passing the `--disabled-debug` flag to the configure command.

To use this feature, enable the `extended_logs` feature of this crate (the linker will fail if your version of `libgphoto2` was not compiled without the `--disabled-debug`).

## Testing

To run the tests of this crate the `test` feature must be enabled:

```sh
cargo test -F test
```

Note that `test` builds a very stripped down version of `libgphoto2`, which is only usable for testing (Don't enable this feature when using this crate).

## Stability

In general all all APIs should be stable, I've tested the ones my camera supported and found no bugs so far.  
If you encounter an error like `BAD_PARAMETERS` or found a bug, please create an issue on [GitHub](https://github.com/maxicarlos08/gphoto2-rs/issues).

## License

Copyright © 2022 Maxicarlos08 <maxicarlos08@gmail.com>

This library uses the `libgphoto2` library, which is
licensed under the [LGPL version 2.1](https://github.com/gphoto/libgphoto2/blob/master/COPYING).

