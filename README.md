# GPhoto2-rs

Rust bindings to libgphoto2

## Features

- [ ] Camera
  - [x] Capture images
  - [x] Download images
  - [x] Get port information
  - [x] Get abilities (model, driver stability, permissions, ...)
  - [x] Read configuration
  - [x] Set configuration
  - [ ] Usb port information (TODO)
  - [ ] Interact with filesystem on camera (TODO)
- [x] Context
  - [x] Autodetect camera
  - [x] Get list of available cameras
  - [x] Get camera by model and port

## Gettings started

### Installation

Run `cargo add gphoto2` to add gphoto2 to your project or add this to your `Cargo.toml`:

```toml
[dependencies]
gphoto2 = "0.1"
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

On MacOs systems with Homebrew run:

```sh
homebrew install libgphoto2
```

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

## License

Copyright © 2022 Maxicarlos08 <maxicarlos08@gmail.com>

This library uses the `libgphoto2` library, which is
licensed under the [LGPL version 2.1](https://github.com/gphoto/libgphoto2/blob/master/COPYING).

