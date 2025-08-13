//! Peer-to-peer networking module for distributed Astor currency operations
//! 
//! Provides node discovery, consensus mechanisms, and network synchronization

pub mod node;
pub mod consensus;
pub mod protocol;
pub mod discovery;
pub mod sync;

pub use node::{AstorNode, NodeConfig, NodeInfo, NodeStatus};
pub use consensus::{ConsensusEngine, ConsensusMessage, ConsensusState};
pub use protocol::{NetworkMessage, MessageType, ProtocolHandler};
pub use discovery::{PeerDiscovery, PeerInfo};
pub use sync::{NetworkSync, SyncManager};

use crate::errors::AstorError;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Network manager that coordinates all networking operations
pub struct NetworkManager {
    pub node: Arc<RwLock<AstorNode>>,
    pub consensus: Arc<RwLock<ConsensusEngine>>,
    pub discovery: Arc<RwLock<PeerDiscovery>>,
    pub sync_manager: Arc<RwLock<SyncManager>>,
    pub protocol_handler: Arc<RwLock<ProtocolHandler>>,
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(config: NodeConfig) -> Result<Self, AstorError> {
        let node = Arc::new(RwLock::new(AstorNode::new(config.clone()).await?));
        let consensus = Arc::new(RwLock::new(ConsensusEngine::new(config.clone()).await?));
        let discovery = Arc::new(RwLock::new(PeerDiscovery::new(config.clone()).await?));
        let sync_manager = Arc::new(RwLock::new(SyncManager::new().await?));
        let protocol_handler = Arc::new(RwLock::new(ProtocolHandler::new().await?));

        Ok(Self {
            node,
            consensus,
            discovery,
            sync_manager,
            protocol_handler,
        })
    }

    /// Start the network services
    pub async fn start(&self) -> Result<(), AstorError> {
        // Start node services
        self.node.write().await.start().await?;
        
        // Start peer discovery
        self.discovery.write().await.start().await?;
        
        // Start consensus engine
        self.consensus.write().await.start().await?;
        
        // Start sync manager
        self.sync_manager.write().await.start().await?;
        
        Ok(())
    }

    /// Stop all network services
    pub async fn stop(&self) -> Result<(), AstorError> {
        self.sync_manager.write().await.stop().await?;
        self.consensus.write().await.stop().await?;
        self.discovery.write().await.stop().await?;
        self.node.write().await.stop().await?;
        Ok(())
    }

    /// Get network status
    pub async fn get_network_status(&self) -> NetworkStatus {
        let node = self.node.read().await;
        let consensus = self.consensus.read().await;
        let discovery = self.discovery.read().await;
        
        NetworkStatus {
            node_id: node.get_id().clone(),
            peer_count: discovery.get_peer_count(),
            consensus_state: consensus.get_state(),
            is_synced: self.sync_manager.read().await.is_synced(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStatus {
    pub node_id: String,
    pub peer_count: usize,
    pub consensus_state: ConsensusState,
    pub is_synced: bool,
}
