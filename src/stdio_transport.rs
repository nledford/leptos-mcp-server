use async_trait::async_trait;
use rust_mcp_sdk::schema::RequestId;
use rust_mcp_sdk::schema::schema_utils::{
    ClientMessage, ClientMessages, MessageFromServer, SdkError, ServerMessage, ServerMessages,
};
use rust_mcp_sdk::{
    IoStream, McpDispatch, MessageDispatcher, Transport, TransportDispatcher, TransportError,
    TransportOptions, TransportResult,
};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::oneshot::Sender;
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;

const CHANNEL_CAPACITY: usize = 36;
const JSONRPC_PARSE_ERROR: i64 = -32700;
const JSONRPC_INVALID_REQUEST: i64 = -32600;

type PendingRequests = std::sync::Arc<Mutex<HashMap<RequestId, Sender<ClientMessage>>>>;
type WriteAck = oneshot::Sender<TransportResult<()>>;
type WriteQueue = mpsc::Sender<(String, WriteAck)>;

pub struct SanitizedStdioTransport {
    options: TransportOptions,
    is_shut_down: Mutex<bool>,
    message_sender: std::sync::Arc<RwLock<Option<MessageDispatcher<ClientMessage>>>>,
    error_stream: RwLock<Option<IoStream>>,
    pending_requests: PendingRequests,
}

impl SanitizedStdioTransport {
    pub fn new(options: TransportOptions) -> Self {
        Self {
            options,
            is_shut_down: Mutex::new(false),
            message_sender: std::sync::Arc::new(RwLock::new(None)),
            error_stream: RwLock::new(None),
            pending_requests: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn set_message_sender(&self, sender: MessageDispatcher<ClientMessage>) {
        let mut lock = self.message_sender.write().await;
        *lock = Some(sender);
    }

    async fn set_error_stream(&self, error_stream: IoStream) {
        let mut lock = self.error_stream.write().await;
        *lock = Some(error_stream);
    }
}

#[async_trait]
impl Transport<ClientMessages, MessageFromServer, ClientMessage, ServerMessages, ServerMessage>
    for SanitizedStdioTransport
{
    async fn start(&self) -> TransportResult<ReceiverStream<ClientMessages>>
    where
        MessageDispatcher<ClientMessage>:
            McpDispatch<ClientMessages, ServerMessages, ClientMessage, ServerMessage>,
    {
        {
            let mut is_shut_down = self.is_shut_down.lock().await;
            *is_shut_down = false;
        }

        let (messages_tx, messages_rx) = mpsc::channel::<ClientMessages>(CHANNEL_CAPACITY);
        let (write_tx, write_rx) = mpsc::channel::<(String, WriteAck)>(CHANNEL_CAPACITY);

        spawn_stdout_writer(write_rx);
        spawn_stdin_reader(messages_tx, write_tx.clone());

        let sender = MessageDispatcher::new_with_acknowledgement(
            self.pending_requests.clone(),
            write_tx,
            self.options.timeout,
        );
        self.set_message_sender(sender).await;
        self.set_error_stream(IoStream::Writable(Box::pin(tokio::io::stderr())))
            .await;

        Ok(ReceiverStream::new(messages_rx))
    }

    fn message_sender(&self) -> std::sync::Arc<RwLock<Option<MessageDispatcher<ClientMessage>>>> {
        self.message_sender.clone()
    }

    fn error_stream(&self) -> &RwLock<Option<IoStream>> {
        &self.error_stream
    }

    async fn shut_down(&self) -> TransportResult<()> {
        let mut is_shut_down = self.is_shut_down.lock().await;
        *is_shut_down = true;
        Ok(())
    }

    async fn is_shut_down(&self) -> bool {
        *self.is_shut_down.lock().await
    }

    async fn consume_string_payload(&self, _payload: &str) -> TransportResult<()> {
        Err(TransportError::Internal(
            "Invalid invocation of consume_string_payload() for SanitizedStdioTransport"
                .to_string(),
        ))
    }

    async fn pending_request_tx(&self, request_id: &RequestId) -> Option<Sender<ClientMessage>> {
        let mut pending_requests = self.pending_requests.lock().await;
        pending_requests.remove(request_id)
    }

    async fn keep_alive(
        &self,
        _interval: Duration,
        _disconnect_tx: oneshot::Sender<()>,
    ) -> TransportResult<JoinHandle<()>> {
        Err(TransportError::Internal(
            "Invalid invocation of keep_alive() for SanitizedStdioTransport".to_string(),
        ))
    }
}

#[async_trait]
impl McpDispatch<ClientMessages, ServerMessages, ClientMessage, ServerMessage>
    for SanitizedStdioTransport
{
    async fn send_message(
        &self,
        message: ServerMessages,
        request_timeout: Option<Duration>,
    ) -> TransportResult<Option<ClientMessages>> {
        let sender = self.message_sender.read().await;
        let sender = sender.as_ref().ok_or(SdkError::connection_closed())?;
        sender.send_message(message, request_timeout).await
    }

    async fn send(
        &self,
        message: ServerMessage,
        request_timeout: Option<Duration>,
    ) -> TransportResult<Option<ClientMessage>> {
        let sender = self.message_sender.read().await;
        let sender = sender.as_ref().ok_or(SdkError::connection_closed())?;
        sender.send(message, request_timeout).await
    }

    async fn send_batch(
        &self,
        message: Vec<ServerMessage>,
        request_timeout: Option<Duration>,
    ) -> TransportResult<Option<Vec<ClientMessage>>> {
        let sender = self.message_sender.read().await;
        let sender = sender.as_ref().ok_or(SdkError::connection_closed())?;
        sender.send_batch(message, request_timeout).await
    }

    async fn write_str(&self, payload: &str, skip_store: bool) -> TransportResult<()> {
        let sender = self.message_sender.read().await;
        let sender = sender.as_ref().ok_or(SdkError::connection_closed())?;
        sender.write_str(payload, skip_store).await
    }
}

impl
    TransportDispatcher<
        ClientMessages,
        MessageFromServer,
        ClientMessage,
        ServerMessages,
        ServerMessage,
    > for SanitizedStdioTransport
{
}

fn spawn_stdout_writer(mut write_rx: mpsc::Receiver<(String, WriteAck)>) {
    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();

        while let Some((payload, ack_tx)) = write_rx.recv().await {
            let result = async {
                stdout.write_all(payload.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
                Ok(())
            }
            .await;

            let write_failed = result.is_err();
            let _ = ack_tx.send(result);

            if write_failed {
                break;
            }
        }
    });
}

fn spawn_stdin_reader(messages_tx: mpsc::Sender<ClientMessages>, write_tx: WriteQueue) {
    tokio::spawn(async move {
        let mut lines = BufReader::new(tokio::io::stdin()).lines();

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => match parse_client_messages(&line) {
                    Ok(messages) => {
                        if messages_tx.send(messages).await.is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        if let Err(error) = write_protocol_error(&write_tx, error).await {
                            tracing::error!("failed to write stdio protocol error: {error}");
                            break;
                        }
                    }
                },
                Ok(None) => break,
                Err(error) => {
                    tracing::error!("failed to read stdio frame: {error}");
                    break;
                }
            }
        }
    });
}

fn parse_client_messages(line: &str) -> Result<ClientMessages, ProtocolError> {
    let value = match serde_json::from_str::<Value>(line) {
        Ok(value) => value,
        Err(_) => return Err(ProtocolError::parse_error()),
    };

    serde_json::from_value::<ClientMessages>(value.clone())
        .map_err(|_| ProtocolError::invalid_request(extract_jsonrpc_id(&value)))
}

async fn write_protocol_error(write_tx: &WriteQueue, error: ProtocolError) -> TransportResult<()> {
    let payload = error.to_payload();
    let (ack_tx, ack_rx) = oneshot::channel();

    write_tx
        .send((payload, ack_tx))
        .await
        .map_err(|error| TransportError::Internal(error.to_string()))?;

    ack_rx.await?
}

fn extract_jsonrpc_id(value: &Value) -> Option<Value> {
    match value {
        Value::Object(object) => match object.get("id") {
            Some(Value::String(id)) => Some(Value::String(id.clone())),
            Some(Value::Number(id)) if id.is_i64() || id.is_u64() => {
                Some(Value::Number(id.clone()))
            }
            _ => None,
        },
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ProtocolError {
    code: i64,
    message: &'static str,
    id: Option<Value>,
}

impl ProtocolError {
    fn parse_error() -> Self {
        Self {
            code: JSONRPC_PARSE_ERROR,
            message: "Parse error",
            id: None,
        }
    }

    fn invalid_request(id: Option<Value>) -> Self {
        Self {
            code: JSONRPC_INVALID_REQUEST,
            message: "Invalid request",
            id,
        }
    }

    fn to_payload(&self) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.id.clone().unwrap_or(Value::Null),
            "error": {
                "code": self.code,
                "message": self.message,
            },
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_client_messages_rejects_invalid_json_with_sanitized_parse_error() {
        assert_eq!(
            parse_client_messages("{not json").expect_err("invalid JSON should fail"),
            ProtocolError::parse_error()
        );
    }

    #[test]
    fn parse_client_messages_rejects_invalid_jsonrpc_with_request_id() {
        assert_eq!(
            parse_client_messages(r#"{"jsonrpc":"2.0","id":"bad-1","params":{}}"#)
                .expect_err("invalid JSON-RPC should fail"),
            ProtocolError::invalid_request(Some(Value::String("bad-1".to_string())))
        );
    }

    #[test]
    fn parse_client_messages_accepts_valid_client_messages() {
        let messages =
            parse_client_messages(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#)
                .expect("valid JSON-RPC should parse");

        assert!(matches!(messages, ClientMessages::Single(_)));
    }

    #[test]
    fn protocol_error_payload_uses_null_id_when_no_valid_id_exists() {
        let payload: Value = serde_json::from_str(&ProtocolError::parse_error().to_payload())
            .expect("error payload should be JSON");

        assert_eq!(payload["jsonrpc"], "2.0");
        assert_eq!(payload["id"], Value::Null);
        assert_eq!(payload["error"]["code"], JSONRPC_PARSE_ERROR);
        assert_eq!(payload["error"]["message"], "Parse error");
    }
}
