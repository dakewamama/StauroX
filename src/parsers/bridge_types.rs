use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeType {
    Wormhole,
    Across,
    DeBridge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "instruction", rename_all = "snake_case")]
pub enum BridgeInstruction {
    /// Transfer wrapped tokens from Solana to another chain
    TransferWrapped {
        amount: u64,
        target_chain: u16,
        #[serde(serialize_with = "serialize_hex")]
        recipient: Vec<u8>,
    },
    
    /// Transfer native tokens from Solana to another chain
    TransferNative {
        amount: u64,
        target_chain: u16,
        #[serde(serialize_with = "serialize_hex")]
        recipient: Vec<u8>,
    },
    
    /// Transfer with additional payload data
    TransferWithPayload {
        amount: u64,
        target_chain: u16,
    },
    
    /// Attest a token for bridging (registers new token)
    AttestToken,
    
    /// Complete a transfer by receiving tokens on Solana
    CompleteTransfer {
        #[serde(serialize_with = "serialize_hex")]
        vaa_hash: Vec<u8>,
        is_native: bool,
    },
    
    /// Complete a transfer with payload
    CompleteTransferWithPayload,
    
    /// Wrapped token creation or completion operations
    WrappedTokenOperation {
        operation_type: String,
    },
    
    /// Unknown or unsupported instruction
    Unknown,
}

// Serialize bytes as hex string
fn serialize_hex<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if bytes.is_empty() {
        serializer.serialize_str("0x")
    } else {
        serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    pub bridge_type: BridgeType,
    pub instruction: BridgeInstruction,
}

impl ParsedTransaction {
    pub fn new(bridge_type: BridgeType, instruction: BridgeInstruction) -> Self {
        Self {
            bridge_type,
            instruction,
        }
    }
    
    pub fn bridge_name(&self) -> &str {
        match self.bridge_type {
            BridgeType::Wormhole => "Wormhole",
            BridgeType::Across => "Across Protocol",
            BridgeType::DeBridge => "DeBridge",
        }
    }

    pub fn instruction_name(&self) -> &str {
        match &self.instruction {
            BridgeInstruction::TransferWrapped { .. } => "TransferWrapped",
            BridgeInstruction::TransferNative { .. } => "TransferNative",
            BridgeInstruction::TransferWithPayload { .. } => "TransferWithPayload",
            BridgeInstruction::AttestToken => "AttestToken",
            BridgeInstruction::CompleteTransfer { .. } => "CompleteTransfer",
            BridgeInstruction::CompleteTransferWithPayload => "CompleteTransferWithPayload",
            BridgeInstruction::WrappedTokenOperation { operation_type } => operation_type,
            BridgeInstruction::Unknown => "Unknown",
        }
    }

    pub fn amount(&self) -> Option<u64> {
        match &self.instruction {
            BridgeInstruction::TransferWrapped { amount, .. } => Some(*amount),
            BridgeInstruction::TransferNative { amount, .. } => Some(*amount),
            BridgeInstruction::TransferWithPayload { amount, .. } => Some(*amount),
            _ => None,
        }
    }

    pub fn target_chain(&self) -> Option<u16> {
        match &self.instruction {
            BridgeInstruction::TransferWrapped { target_chain, .. } => Some(*target_chain),
            BridgeInstruction::TransferNative { target_chain, .. } => Some(*target_chain),
            BridgeInstruction::TransferWithPayload { target_chain, .. } => Some(*target_chain),
            _ => None,
        }
    }
    
    pub fn target_chain_name(&self) -> Option<&str> {
        self.target_chain().map(|chain_id| {
            match chain_id {
                1 => "Solana",
                2 => "Ethereum",
                3 => "Terra",
                4 => "BSC",
                5 => "Polygon",
                6 => "Avalanche",
                7 => "Oasis",
                8 => "Algorand",
                9 => "Aurora",
                10 => "Fantom",
                11 => "Karura",
                12 => "Acala",
                13 => "Klaytn",
                14 => "Celo",
                15 => "Near",
                16 => "Moonbeam",
                17 => "Neon",
                18 => "Terra2",
                19 => "Injective",
                20 => "Osmosis",
                21 => "Sui",
                22 => "Aptos",
                23 => "Arbitrum",
                24 => "Optimism",
                25 => "Gnosis",
                26 => "Pythnet",
                28 => "Xpla",
                29 => "BTC",
                30 => "Base",
                32 => "Sei",
                33 => "Rootstock",
                34 => "Scroll",
                35 => "Mantle",
                36 => "Blast",
                37 => "Xlayer",
                38 => "Linea",
                39 => "Berachain",
                40 => "Seievm",
                41 => "Cosmoshub",
                42 => "Evmos",
                43 => "Kujira",
                44 => "Neutron",
                45 => "Celestia",
                46 => "Stargaze",
                47 => "Seda",
                48 => "Dymension",
                49 => "Provenance",
                50 => "Sepolia",
                4000 => "PolygonSepolia",
                10002 => "BaseSepolia",
                10003 => "OptimismSepolia",
                10004 => "HoleskyTestnet",
                10005 => "ArbitrumSepolia",
                _ => "Unknown",
            }
        })
    }

    pub fn recipient(&self) -> Option<&[u8]> {
        match &self.instruction {
            BridgeInstruction::TransferWrapped { recipient, .. } => Some(recipient),
            BridgeInstruction::TransferNative { recipient, .. } => Some(recipient),
            _ => None,
        }
    }

    pub fn vaa_hash(&self) -> Option<&[u8]> {
        match &self.instruction {
            BridgeInstruction::CompleteTransfer { vaa_hash, .. } => Some(vaa_hash),
            _ => None,
        }
    }

    pub fn is_outbound(&self) -> bool {
        matches!(
            &self.instruction,
            BridgeInstruction::TransferWrapped { .. }
                | BridgeInstruction::TransferNative { .. }
                | BridgeInstruction::TransferWithPayload { .. }
        )
    }

    pub fn is_inbound(&self) -> bool {
        matches!(
            &self.instruction,
            BridgeInstruction::CompleteTransfer { .. }
                | BridgeInstruction::CompleteTransferWithPayload
        )
    }

    pub fn is_token_operation(&self) -> bool {
        matches!(
            &self.instruction,
            BridgeInstruction::AttestToken
                | BridgeInstruction::WrappedTokenOperation { .. }
        )
    }

    pub fn direction(&self) -> &str {
        if self.is_outbound() {
            "Outbound"
        } else if self.is_inbound() {
            "Inbound"
        } else if self.is_token_operation() {
            "Token Operation"
        } else {
            "Unknown"
        }
    }
}

impl std::fmt::Display for ParsedTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            self.bridge_name(),
            self.instruction_name()
        )?;
        
        if let Some(amount) = self.amount() {
            write!(f, " ({} lamports)", amount)?;
        }
        
        if let Some(chain) = self.target_chain_name() {
            write!(f, " → {}", chain)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_wrapped() {
        let instruction = BridgeInstruction::TransferWrapped {
            amount: 1_000_000,
            target_chain: 2,
            recipient: vec![0x12, 0x34],
        };
        
        let tx = ParsedTransaction::new(BridgeType::Wormhole, instruction);
        
        assert_eq!(tx.instruction_name(), "TransferWrapped");
        assert_eq!(tx.amount(), Some(1_000_000));
        assert_eq!(tx.target_chain(), Some(2));
        assert_eq!(tx.target_chain_name(), Some("Ethereum"));
        assert!(tx.is_outbound());
        assert!(!tx.is_inbound());
    }

    #[test]
    fn test_complete_transfer() {
        let instruction = BridgeInstruction::CompleteTransfer {
            vaa_hash: vec![],
            is_native: false,
        };
        
        let tx = ParsedTransaction::new(BridgeType::Wormhole, instruction);
        
        assert_eq!(tx.instruction_name(), "CompleteTransfer");
        assert_eq!(tx.amount(), None);
        assert!(!tx.is_outbound());
        assert!(tx.is_inbound());
    }

    #[test]
    fn test_attest_token() {
        let instruction = BridgeInstruction::AttestToken;
        let tx = ParsedTransaction::new(BridgeType::Wormhole, instruction);
        
        assert_eq!(tx.instruction_name(), "AttestToken");
        assert!(tx.is_token_operation());
        assert_eq!(tx.direction(), "Token Operation");
    }

    #[test]
    fn test_display() {
        let instruction = BridgeInstruction::TransferWrapped {
            amount: 1_000_000,
            target_chain: 30,
            recipient: vec![],
        };
        
        let tx = ParsedTransaction::new(BridgeType::Wormhole, instruction);
        let display = format!("{}", tx);
        
        assert!(display.contains("Wormhole"));
        assert!(display.contains("TransferWrapped"));
        assert!(display.contains("1000000"));
        assert!(display.contains("Base"));
    }
}