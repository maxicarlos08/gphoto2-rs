use gphoto2::Context;

fn main() {
  let context = Context::new().expect("Failed to create camera");
  let camera = context.autodetect_camera().expect("Failed to auto detect camera");

  println!("Camera summray: {}", camera.summary().unwrap());
}
