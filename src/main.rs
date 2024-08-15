use concord4::Concord4;
use tokio::{
  sync::{broadcast, mpsc},
  task::JoinSet,
};
use tokio_util::sync::CancellationToken;

mod config;
mod monitoring;
mod websocket;

#[tokio::main]
async fn main() {
  monitoring::init_logger();

  let config = config::Concord4HAConfig::new();

  tracing::info!("initializing concord4 connection");
  let mut concord = Concord4::open(&config.serial_device)
    .await
    .expect("failed to open concord4 connection");

  tracing::info!("starting websocket server");

  let addr = format!("0.0.0.0:{}", config.socket_port);
  let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind");
  tracing::info!("Listening on: {}", addr);

  let mut threads = JoinSet::new();
  let cancel_token = CancellationToken::new();

  let (state_tx, _) = broadcast::channel(16);
  let (panel_tx, mut panel_rx) = mpsc::channel(16);

  loop {
    tokio::select! {
      Some(msg) = panel_rx.recv() => {
        if let Err(err) = concord.send(msg).await {
          tracing::error!("error sending message to panel: {:?}", err);
        }
      }
      msg = concord.recv() => {
        match msg {
          Some(Ok(msg)) => {
            // this is a broadcast, so we don't care if it fails, and it will fail if there are no listeners
            let _ = state_tx.send(msg);
          }
          Some(Err(err)) => {
            tracing::error!("error receiving message from panel: {:?}", err);
          }
          None => {
            tracing::error!("FATAL: panel connection closed");
            break;
          }
        }
      }
      Ok((stream, _)) = listener.accept() =>  {
        threads.spawn(websocket::accept_connection(stream, concord.state.clone(), (panel_tx.clone(), state_tx.subscribe()), cancel_token.clone()));
      }
      Some(Err(err)) = threads.join_next() => {
        tracing::error!("error on thread: {:?}", err);
      }
      _ = monitoring::wait_for_signal() => break,
    }
  }

  tracing::info!("shutting down websocket server");
  cancel_token.cancel();

  tracing::info!("waiting for threads to finish");
  loop {
    let thread = threads.join_next().await;
    if thread.is_none() {
      break;
    }

    if let Err(err) = thread.expect("infallible") {
      tracing::error!("error on thread: {:?}", err);
    }
  }
}
