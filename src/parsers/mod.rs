pub mod bridge_types;
pub mod wormhole;

pub use bridge_types::{BridgeInstruction, BridgeType, ParsedTransaction};

use crate::error::{Result, StauroXError};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use std::str::FromStr;
use tracing::info;

// Main parser that detects bridge type and extracts instruction data
pub struct TransactionParser;

impl TransactionParser {
    pub fn new() -> Self {
        Self
    }

    // Parse a transaction and extract bridge instruction details
    pub fn parse_transaction(
        &self,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<Option<ParsedTransaction>> {
        // Extract program IDs from transaction
        let program_ids = self.extract_program_ids(tx)?;
        
        info!("Found {} program IDs in transaction", program_ids.len());
        
        // Detect bridge type
        let bridge_type = self.detect_bridge_type(&program_ids)?;
        
        if bridge_type.is_none() {
            info!("No bridge program detected");
            return Ok(None);
        }

        info!("Detected bridge: {:?}", bridge_type);

        // Parse specific bridge instruction
        let instruction = self.parse_bridge_instruction(tx, bridge_type.unwrap())?;
        
        Ok(Some(ParsedTransaction {
            bridge_type: bridge_type.unwrap(),
            instruction,
        }))
    }

    /// Extract program IDs from transaction
    fn extract_program_ids(
        &self,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<Vec<Pubkey>> {
        let account_keys = match &tx.transaction.transaction {
            solana_transaction_status::EncodedTransaction::Json(ui_tx) => {
                match &ui_tx.message {
                    solana_transaction_status::UiMessage::Raw(msg) => {
                        msg.account_keys.iter()
                            .filter_map(|key| Pubkey::from_str(key).ok())
                            .collect()
                    }
                    solana_transaction_status::UiMessage::Parsed(_) => {
                        return Err(StauroXError::verification(
                            "Parsed message format not supported"
                        ));
                    }
                }
            }
            _ => {
                return Err(StauroXError::verification(
                    "Unsupported transaction encoding - expected JSON"
                ));
            }
        };
        
        Ok(account_keys)
    }

    // Detect which bridge protocol was used
    fn detect_bridge_type(&self, program_ids: &[Pubkey]) -> Result<Option<BridgeType>> {
        const WORMHOLE_TOKEN_BRIDGE: &str = "wormDTUJ6AWPNvk59vGQbDvGJmqbDTdgWgAqcLBCgUb";
        const WORMHOLE_CORE: &str = "worm2ZoG2kUd4vFXhvjh93UUH596ayRfgQ2MgjNMTth";
        
        for id in program_ids {
            let id_str = id.to_string();
            
            if id_str == WORMHOLE_TOKEN_BRIDGE || id_str == WORMHOLE_CORE {
                info!("✓ Detected Wormhole program: {}", id_str);
                return Ok(Some(BridgeType::Wormhole));
            }
            // Add more bridges here
        }
        
        Ok(None)
    }

    // Parse bridge-specific instruction data
    fn parse_bridge_instruction(
        &self,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
        bridge_type: BridgeType,
    ) -> Result<BridgeInstruction> {
        match bridge_type {
            BridgeType::Wormhole => wormhole::parse_wormhole_instruction(tx),
            BridgeType::Across => Ok(BridgeInstruction::Unknown),
            BridgeType::DeBridge => Ok(BridgeInstruction::Unknown),
        }
    }
}

impl Default for TransactionParser {
    fn default() -> Self {
        Self::new()
    }
}