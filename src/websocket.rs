use concord4::{ConcordState, ConcordStateInner, RecvMessage as ConcordMessage, SendableMessage as ConcordCommand};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
  net::TcpStream,
  sync::{broadcast, mpsc},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tokio_util::sync::CancellationToken;

pub async fn accept_connection(
  stream: TcpStream,
  state: ConcordState,
  (concord_tx, mut concord_rx): (mpsc::Sender<ConcordCommand>, broadcast::Receiver<ConcordMessage>),
  cancel_token: CancellationToken,
) -> Result<(), anyhow::Error> {
  let addr = stream.peer_addr()?;
  let mut ws = Websocket::accept(stream).await?;
  tracing::info!("new websocket connection: {}", addr);

  ws.send_state(&state).await?;

  loop {
    tokio::select! {
      msg = ws.recv() => {
        match msg {
          Ok(Some(msg)) => match msg {
            ReceivableMessage::GetState => {
              ws.send_state(&state).await?;
            },
            ReceivableMessage::Command(command) => {
              tracing::debug!("received command: {:?}", command);
              concord_tx.send(command).await?;
            }
          },
          Err(err) => {
            tracing::error!("invalid message received: {}", err);
          }
          Ok(None) => {
            tracing::info!("websocket connection closed: {}", addr);
            break;
          }
        }
      }
      Ok(data) = concord_rx.recv() => {
        match data {
          // client doesn't care about these
          ConcordMessage::Ack | ConcordMessage::Nak => {}
          // useless messages (they are for real alarm panels)
          ConcordMessage::SirenSync | ConcordMessage::Touchpad(_) => {}
          // prevent leaking your alarm code
          ConcordMessage::UserData(_) => {}
          // need to resend the full state when this data is populated
          ConcordMessage::EqptListDone | ConcordMessage::TimeAndDate(_) => {
            ws.send_state(&state).await?;
          }
          // can forward anything else
          msg => {
            ws.send_msg(&msg).await?;
          }

        }
      }
      _ = cancel_token.cancelled() => {
        tracing::info!("gracefully shutting down websocket connection: {}", addr);
        ws.close().await?;
        tracing::info!("closed websocket connection: {}", addr);

        break;
      }
    }
  }

  Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
enum ReceivableMessage {
  Command(ConcordCommand),
  GetState,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
enum SendableMessage<'a> {
  State(&'a ConcordStateInner),
  Message(&'a ConcordMessage),
}

#[derive(Debug)]
struct Websocket(WebSocketStream<TcpStream>);

impl Websocket {
  async fn accept(stream: TcpStream) -> Result<Self, anyhow::Error> {
    Ok(Websocket(tokio_tungstenite::accept_async(stream).await?))
  }

  async fn close(&mut self) -> Result<(), anyhow::Error> {
    self.0.close(None).await?;
    Ok(())
  }

  async fn send<'a>(&mut self, msg: SendableMessage<'a>) -> Result<(), anyhow::Error> {
    let text = serde_json::to_string(&msg)?;
    tracing::trace!(target: "concord4ws::websocket::communication", "sending message: {}", text);

    self.0.send(Message::Text(text)).await?;
    Ok(())
  }

  async fn send_state(&mut self, state: &ConcordState) -> Result<(), anyhow::Error> {
    self.send(SendableMessage::State(state.as_ref())).await
  }

  async fn send_msg(&mut self, msg: &ConcordMessage) -> Result<(), anyhow::Error> {
    self.send(SendableMessage::Message(msg)).await
  }

  async fn recv(&mut self) -> Result<Option<ReceivableMessage>, anyhow::Error> {
    loop {
      match self.0.next().await {
        Some(Ok(Message::Text(text))) => return Ok(Some(serde_json::from_str(&text)?)),
        Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
        Some(Ok(Message::Close(_))) | Some(Err(_)) | None => return Ok(None),
        Some(Ok(msg)) => {
          tracing::debug!(target: "concord4ws::websocket::communication", "received non-text message: {:?}", msg);
        }
      }
    }
  }
}
