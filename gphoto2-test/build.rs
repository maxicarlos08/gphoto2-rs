fn main() {
  assert!(std::process::Command::new("sh").arg("build.sh").status().unwrap().success());
}
