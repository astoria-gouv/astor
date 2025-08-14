//! Network protocol definitions and message handling

use crate::errors::AstorError;
use crate::ledger::Transaction;
use crate::security::Signature;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Handshake,
    Transaction,
    Block,
    Consensus,
    Sync,
    Ping,
    Pong,
    PeerDiscovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub id: String,
    pub message_type: MessageType,
    pub from: String,
    pub to: Option<String>,
    pub payload: MessagePayload,
    pub timestamp: u64,
    pub signature: Option<Signature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Handshake {
        node_id: String,
        version: String,
        capabilities: Vec<String>,
        public_key: Vec<u8>,
    },
    Transaction {
        transaction: Transaction,
    },
    Block {
        block_data: Vec<u8>,
    },
    Consensus {
        consensus_data: Vec<u8>,
    },
    Sync {
        request_type: SyncRequestType,
        data: Vec<u8>,
    },
    Ping {
        nonce: u64,
    },
    Pong {
        nonce: u64,
    },
    PeerDiscovery {
        peers: Vec<PeerInfo>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequestType {
    GetBlocks,
    GetTransactions,
    GetState,
    BlockResponse,
    TransactionResponse,
    StateResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub public_key: Vec<u8>,
    pub last_seen: u64,
}

pub struct ProtocolHandler {
    message_handlers: HashMap<MessageType, Box<dyn MessageHandler + Send + Sync>>,
    outbound_sender: mpsc::UnboundedSender<NetworkMessage>,
    inbound_receiver: Option<mpsc::UnboundedReceiver<NetworkMessage>>,
}

pub trait MessageHandler {
    async fn handle(&self, message: NetworkMessage) -> Result<Option<NetworkMessage>, AstorError>;
}

impl ProtocolHandler {
    pub async fn new() -> Result<Self, AstorError> {
        let (outbound_sender, _outbound_receiver) = mpsc::unbounded_channel();
        let (_inbound_sender, inbound_receiver) = mpsc::unbounded_channel();

        let mut handler = Self {
            message_handlers: HashMap::new(),
            outbound_sender,
            inbound_receiver: Some(inbound_receiver),
        };

        // Register default message handlers
        handler.register_handlers().await?;

        Ok(handler)
    }

    async fn register_handlers(&mut self) -> Result<(), AstorError> {
        // Register handlers for different message types
        self.message_handlers
            .insert(MessageType::Handshake, Box::new(HandshakeHandler::new()));
        self.message_handlers.insert(
            MessageType::Transaction,
            Box::new(TransactionHandler::new()),
        );
        self.message_handlers
            .insert(MessageType::Ping, Box::new(PingHandler::new()));

        Ok(())
    }

    pub async fn handle_message(&self, message: NetworkMessage) -> Result<(), AstorError> {
        if let Some(handler) = self.message_handlers.get(&message.message_type) {
            if let Some(response) = handler.handle(message).await? {
                self.send_message(response).await?;
            }
        }
        Ok(())
    }

    pub async fn send_message(&self, message: NetworkMessage) -> Result<(), AstorError> {
        self.outbound_sender
            .send(message)
            .map_err(|e| AstorError::NetworkError(format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    pub fn create_message(
        from: String,
        to: Option<String>,
        message_type: MessageType,
        payload: MessagePayload,
    ) -> NetworkMessage {
        NetworkMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message_type,
            from,
            to,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: None,
        }
    }
}

// Message handler implementations
struct HandshakeHandler;

impl HandshakeHandler {
    fn new() -> Self {
        Self
    }
}

impl MessageHandler for HandshakeHandler {
    async fn handle(&self, message: NetworkMessage) -> Result<Option<NetworkMessage>, AstorError> {
        match message.payload {
            MessagePayload::Handshake { .. } => {
                // Process handshake and return response
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

struct TransactionHandler;

impl TransactionHandler {
    fn new() -> Self {
        Self
    }
}

impl MessageHandler for TransactionHandler {
    async fn handle(&self, message: NetworkMessage) -> Result<Option<NetworkMessage>, AstorError> {
        match message.payload {
            MessagePayload::Transaction { transaction } => {
                // Process transaction
                tracing::info!("Received transaction: {:?}", transaction);
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

struct PingHandler;

impl PingHandler {
    fn new() -> Self {
        Self
    }
}

impl MessageHandler for PingHandler {
    async fn handle(&self, message: NetworkMessage) -> Result<Option<NetworkMessage>, AstorError> {
        match message.payload {
            MessagePayload::Ping { nonce } => {
                // Return pong response
                let response = ProtocolHandler::create_message(
                    "self".to_string(),
                    Some(message.from),
                    MessageType::Pong,
                    MessagePayload::Pong { nonce },
                );
                Ok(Some(response))
            }
            _ => Ok(None),
        }
    }
}
