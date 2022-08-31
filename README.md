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
use gphoto2::Context;
use std::path::Path;

fn main() {
  // Everything starts from a context
  let context = Context::new().expect("Failed to create context");
  // From the context you can detect cameras
  let camera = context.autodetect_camera().expect("Failed to autodetect camera");

  // And take pictures
  let file_path = camera.capture_image().expect("Could not capture image");
  file_path
    .download(&camera, Path::new(&file_path.name().to_string()))
    .expect("Failed to download image");

  // For more advanced examples take a look at the examples/ foldeer
}
```

You can find more examples [here](https://github.com/maxicarlos08/gphoto2-rs/tree/master/examples)

## Stability

In general all all APIs should be stable, I've tested the ones my camera supported and found no bugs so far.  
If you encounter an error like `BAD_PARAMETERS` or found a bug, please create an issue on [GitHub](https://github.com/maxicarlos08/gphoto2-rs/issues).

## License

Copyright © 2022 Maxicarlos08 <maxicarlos08@gmail.com>

This library uses the `libgphoto2` library, which is
licensed under the [LGPL version 2.1](https://github.com/gphoto/libgphoto2/blob/master/COPYING).

