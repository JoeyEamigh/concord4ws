#[derive(Debug, Clone)]
pub struct Concord4HAConfig {
  pub serial_device: String,
  pub socket_port: u16,
}

impl Concord4HAConfig {
  pub fn new() -> Self {
    #[cfg(feature = "dotenv")]
    dotenv::dotenv().ok();

    let serial_device = std::env::var("SERIAL_DEVICE").expect("serial device is required");
    let socket_port = std::env::var("SOCKET_PORT")
      .unwrap_or("8080".to_string())
      .parse::<u16>()
      .unwrap_or(8080);

    Self {
      serial_device,
      socket_port,
    }
  }
}
