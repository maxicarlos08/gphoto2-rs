use std::env;
use std::path::PathBuf;

fn main() {
  let lib = pkg_config::Config::new()
    .atleast_version("2.5.10")
    .probe("libgphoto2")
    .expect("Could not find libgphoto2");

  let bindings = bindgen::Builder::default()
    .clang_args(lib.include_paths.iter().map(|path| format!("-I{}", path.to_string_lossy())))
    .header("src/wrapper.h")
    .generate_comments(true)
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
    .bitfield_enum("^Camera(FilePermissions|(File|Folder)?Operation)$")
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
}
