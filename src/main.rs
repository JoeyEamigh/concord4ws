use concord4::Concord4;
use tracing::info;

mod config;

#[tokio::main]
async fn main() {
  init_logger();

  let config = config::Concord4HAConfig::new();

  info!("initializing concord4 connection");
  let concord = Concord4::init(&config.serial_device);

  info!("waiting for concord4 to be ready");
  tokio::select! {
    _ = concord.wait_ready() => {
      info!("concord4-ha is ready");
    }
    _ = concord.block() => {
      info!("received control-c, exiting...");
      return;
    }
  }

  info!("starting websocket server");

  let addr = format!("0.0.0.0:{}", config.socket_port);
  let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind");
  info!("Listening on: {}", addr);

  loop {
    tokio::select! {
      Ok((stream, _)) = listener.accept() =>  {
        tokio::spawn(accept_connection(stream, concord.clone()));
      }
      _ = concord.block() => {
        info!("received control-c, exiting...");
        break;
      }
    }
  }
}

async fn accept_connection(stream: tokio::net::TcpStream, concord: Concord4) {
  use futures::{SinkExt, StreamExt};
  use tokio_tungstenite::tungstenite::Message;

  let (_concord_tx, mut concord_rx) = concord.subscribe();

  let addr = stream
    .peer_addr()
    .expect("connected streams should have a peer address");
  info!("peer address: {}", addr);

  let ws_stream = tokio_tungstenite::accept_async(stream)
    .await
    .expect("Error during the websocket handshake occurred");

  info!("new websocket connection: {}", addr);

  let (mut write, mut read) = ws_stream.split();

  if let Ok(data) = concord.data().await.to_json() {
    let _ = write.send(Message::Text(data)).await;
  }

  loop {
    tokio::select! {
      Some(msg) = read.next() => {
        if let Ok(msg) = msg {
          match msg {
            Message::Text(text) => {
              info!("received message: {}", text);
            }
            Message::Close(_) => {
              info!("closing websocket connection: {}", addr);
              break;
            }
            _ => {}
          }
        }

      }
      Ok(data) = concord_rx.recv() => {
        if let Ok(data) = data.to_json() {
          let _ = write.send(Message::Text(data)).await;
        }
      }
    }
  }
}

fn init_logger() {
  use tracing::metadata::LevelFilter;
  use tracing_subscriber::{
    filter::Directive, fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
  };

  // directives for debug builds
  #[cfg(debug_assertions)]
  let default_directive = Directive::from(LevelFilter::TRACE);

  #[cfg(debug_assertions)]
  let filter_directives = if let Ok(filter) = std::env::var("RUST_LOG") {
    filter
  } else {
    "concord4-ha=trace,concord4=trace,concord4::serial=info,concord4::siren-sync=info,concord4::touchpad=info"
      .to_string()
  };

  // directives for release builds
  #[cfg(not(debug_assertions))]
  let default_directive = Directive::from(LevelFilter::INFO);

  #[cfg(not(debug_assertions))]
  let filter_directives = if let Ok(filter) = std::env::var("RUST_LOG") {
    filter
  } else {
    "concord4-ha=info,concord4=info".to_string()
  };

  let filter = EnvFilter::builder()
    .with_default_directive(default_directive)
    .parse_lossy(filter_directives);

  tracing_subscriber::registry()
    .with(fmt::layer().with_filter(filter))
    .init();
}
