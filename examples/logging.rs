use tracing_subscriber::{prelude::*, EnvFilter};

pub fn setup() {
  tracing_subscriber::registry().with(tracing_subscriber::fmt::layer()).init();
}

#[allow(dead_code)]
fn main() {
  eprintln!("This is only a library file")
}
