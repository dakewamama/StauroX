use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::error::Result;
use super::consensus::ConsensusEngine;

/// Multi-RPC client with consensus verification
pub struct MultiRpcClient {
    clients: Vec<Arc<RpcClient>>,
    consensus: ConsensusEngine,
}

impl MultiRpcClient {
    pub fn new(rpc_urls: Vec<String>, consensus_threshold: usize) -> Self {
        let clients: Vec<_> = rpc_urls
            .into_iter()
            .map(|url| Arc::new(RpcClient::new(url)))
            .collect();

        let consensus = ConsensusEngine::new(consensus_threshold, clients.len());

        Self { clients, consensus }
    }

    /// Fetch transaction from multiple RPCs with consensus
    pub async fn fetch_transaction_with_consensus(
        &self,
        signature: &Signature,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
        let signature = *signature;  // FIXED: Copy signature to avoid lifetime issues
        
        let results = self.fetch_from_all_rpcs(move |client| {
            client.get_transaction(&signature, UiTransactionEncoding::Json)
        }).await;

        self.consensus.has_minimum_responses(&results)?;
        
        debug!(
            "Transaction consensus: {}/{} RPCs responded",
            results.len(),
            self.clients.len()
        );

        // Return first result
        results.into_iter().next().ok_or_else(|| {
            crate::error::StauroXError::consensus_failure(0, self.consensus.threshold())
        })
    }

    /// Get current slot from multiple RPCs with consensus
    pub async fn get_slot_with_consensus(&self) -> Result<u64> {
        let slots = self.fetch_from_all_rpcs(|client| client.get_slot()).await;

        let consensus_slot = self.consensus.find_consensus(slots)?;

        debug!("Slot consensus achieved: {}", consensus_slot);
        Ok(consensus_slot)
    }

    /// Generic method to fetch from all RPCs in parallel
    async fn fetch_from_all_rpcs<T, F>(&self, fetch_fn: F) -> Vec<T>
    where
        T: Send + 'static,
        F: Fn(&RpcClient) -> solana_client::client_error::Result<T> + Send + 'static + Clone,
    {
        let mut handles = vec![];

        for (idx, client) in self.clients.iter().enumerate() {
            let client = Arc::clone(client);
            let fetch_fn = fetch_fn.clone();

            let handle = tokio::spawn(async move {
                match fetch_fn(&client) {
                    Ok(result) => Some((idx, result)),
                    Err(_e) => {
                        //warn!("RPC {} failed: {}", idx, e);
                        None
                    }
                }
            });

            handles.push(handle);
        }

        // Collect successful results
        let mut results = vec![];
        for handle in handles {
            if let Ok(Some((idx, result))) = handle.await {
                debug!("RPC {} responded successfully", idx);
                results.push(result);
            }
        }

        results
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn consensus_threshold(&self) -> usize {
        self.consensus.threshold()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_rpc_creation() {
        let rpcs = vec![
            "https://api.mainnet-beta.solana.com".to_string(),
            "https://rpc.ankr.com/solana".to_string(),
        ];

        let client = MultiRpcClient::new(rpcs, 2);
        assert_eq!(client.client_count(), 2);
        assert_eq!(client.consensus_threshold(), 2);
    }
}