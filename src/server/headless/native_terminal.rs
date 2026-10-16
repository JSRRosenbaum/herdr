use super::*;
use std::sync::mpsc::TrySendError;

impl HeadlessServer {
    /// Drain ordered PTY chunks without putting byte streams on the droppable
    /// render lane or the unbounded priority control lane. A slow client keeps
    /// at most one framed chunk here and backpressures the PTY's bounded channel.
    pub(super) fn stream_native_output(&mut self) {
        let mut broken = Vec::new();
        for (&client_id, client) in &mut self.clients {
            let Some(receiver) = client.native_output.as_mut() else {
                continue;
            };
            let Some(writer) = client.writer.as_ref() else {
                broken.push(client_id);
                continue;
            };
            for _ in 0..32 {
                let framed = if let Some(pending) = client.native_pending.take() {
                    pending
                } else {
                    let bytes = match receiver.try_recv() {
                        Ok(bytes) => bytes,
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            broken.push(client_id);
                            break;
                        }
                    };
                    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
                    let message = ServerMessage::EndpointControl {
                        kind: crate::server::terminal_attach::NATIVE_BYTES_KIND.into(),
                        data: encoded,
                    };
                    match Self::frame_server_message(&message) {
                        Ok(framed) => framed,
                        Err(error) => {
                            warn!(client_id, %error, "native terminal output could not be framed");
                            broken.push(client_id);
                            break;
                        }
                    }
                };
                match writer.render.send_ordered(framed) {
                    Ok(()) => {}
                    Err(TrySendError::Full(framed)) => {
                        client.native_pending = Some(framed);
                        break;
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        broken.push(client_id);
                        break;
                    }
                }
            }
        }
        for client_id in broken {
            self.remove_client_and_resize_if_needed(client_id);
        }
    }

    pub(super) fn native_output_waiting(&self) -> bool {
        self.clients.values().any(|client| {
            client.native_pending.is_some()
                || client
                    .native_output
                    .as_ref()
                    .is_some_and(|receiver| !receiver.is_empty())
        })
    }
}
