fn main() {
  assert!(std::process::Command::new("sh").args(&["-c", "./build.sh"]).status().unwrap().success());
}
