// Copyright 2019, The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::{sync::Arc, time::Duration};

use tari_comms::{connectivity::ConnectivityRequester, NodeIdentity, PeerManager};
use tari_shutdown::ShutdownSignal;
use tokio::sync::mpsc;

use crate::{
    dht::DhtInitializationError,
    outbound::DhtOutboundRequest,
    version::DhtProtocolVersion,
    DbConnectionUrl,
    Dht,
    DhtConfig,
};

#[derive(Debug, Clone, Default)]
pub struct DhtBuilder {
    config: DhtConfig,
    outbound_tx: Option<mpsc::Sender<DhtOutboundRequest>>,
}

impl DhtBuilder {
    pub fn new() -> Self {
        Self {
            #[cfg(test)]
            config: DhtConfig::default_local_test(),
            #[cfg(not(test))]
            config: Default::default(),
            outbound_tx: None,
        }
    }

    pub fn with_config(&mut self, config: DhtConfig) -> &mut Self {
        self.config = config;
        self
    }

    pub fn local_test(&mut self) -> &mut Self {
        self.config = DhtConfig::default_local_test();
        self
    }

    pub fn with_protocol_version(&mut self, protocol_version: DhtProtocolVersion) -> &mut Self {
        self.config.protocol_version = protocol_version;
        self
    }

    pub fn set_auto_store_and_forward_requests(&mut self, enabled: bool) -> &mut Self {
        self.config.saf_config.auto_request = enabled;
        self
    }

    pub fn with_outbound_sender(&mut self, outbound_tx: mpsc::Sender<DhtOutboundRequest>) -> &mut Self {
        self.outbound_tx = Some(outbound_tx);
        self
    }

    pub fn testnet(&mut self) -> &mut Self {
        self.config = DhtConfig::default_testnet();
        self
    }

    pub fn mainnet(&mut self) -> &mut Self {
        self.config = DhtConfig::default_mainnet();
        self
    }

    pub fn with_database_url(&mut self, database_url: DbConnectionUrl) -> &mut Self {
        self.config.database_url = database_url;
        self
    }

    pub fn with_dedup_cache_trim_interval(&mut self, trim_interval: Duration) -> &mut Self {
        self.config.dedup_cache_trim_interval = trim_interval;
        self
    }

    pub fn with_dedup_cache_capacity(&mut self, capacity: usize) -> &mut Self {
        self.config.dedup_cache_capacity = capacity;
        self
    }

    pub fn with_dedup_discard_hit_count(&mut self, max_hit_count: usize) -> &mut Self {
        self.config.dedup_allowed_message_occurrences = max_hit_count;
        self
    }

    pub fn with_num_random_nodes(&mut self, n: usize) -> &mut Self {
        self.config.num_random_nodes = n;
        self
    }

    pub fn with_num_neighbouring_nodes(&mut self, n: usize) -> &mut Self {
        self.config.num_neighbouring_nodes = n;
        self.config.saf_config.num_neighbouring_nodes = n;
        self
    }

    pub fn with_propagation_factor(&mut self, propagation_factor: usize) -> &mut Self {
        self.config.propagation_factor = propagation_factor;
        self
    }

    pub fn with_broadcast_factor(&mut self, broadcast_factor: usize) -> &mut Self {
        self.config.broadcast_factor = broadcast_factor;
        self
    }

    pub fn with_discovery_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.config.discovery_request_timeout = timeout;
        self
    }

    pub fn enable_auto_join(&mut self) -> &mut Self {
        self.config.auto_join = true;
        self
    }

    /// Build and initialize a Dht object.
    ///
    /// Will panic if not in a tokio runtime context
    pub async fn build(
        &mut self,
        node_identity: Arc<NodeIdentity>,
        peer_manager: Arc<PeerManager>,
        connectivity: ConnectivityRequester,
        shutdown_signal: ShutdownSignal,
    ) -> Result<Dht, DhtInitializationError> {
        let outbound_tx = self
            .outbound_tx
            .take()
            .ok_or(DhtInitializationError::BuilderNoOutboundMessageSender)?;

        Dht::initialize(
            self.config.clone(),
            node_identity,
            peer_manager,
            outbound_tx,
            connectivity,
            shutdown_signal,
        )
        .await
    }
}
