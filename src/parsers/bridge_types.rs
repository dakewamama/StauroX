use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeType {
    Wormhole,
    Across,
    DeBridge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeInstruction {
    TransferWrapped {
        amount: u64,
        target_chain: u16,
        #[serde(serialize_with = "serialize_hex")]
        recipient: Vec<u8>,
    },
    TransferNative {
        amount: u64,
        target_chain: u16,
        #[serde(serialize_with = "serialize_hex")]
        recipient: Vec<u8>,
    },
    Unknown,
}

// Serialize bytes as hex string
fn serialize_hex<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    pub bridge_type: BridgeType,
    pub instruction: BridgeInstruction,
}

impl ParsedTransaction {
    pub fn bridge_name(&self) -> &str {
        match self.bridge_type {
            BridgeType::Wormhole => "Wormhole",
            BridgeType::Across => "Across Protocol",
            BridgeType::DeBridge => "DeBridge",
        }
    }

    pub fn amount(&self) -> Option<u64> {
        match &self.instruction {
            BridgeInstruction::TransferWrapped { amount, .. } => Some(*amount),
            BridgeInstruction::TransferNative { amount, .. } => Some(*amount),
            BridgeInstruction::Unknown => None,
        }
    }

    pub fn target_chain(&self) -> Option<u16> {
        match &self.instruction {
            BridgeInstruction::TransferWrapped { target_chain, .. } => Some(*target_chain),
            BridgeInstruction::TransferNative { target_chain, .. } => Some(*target_chain),
            BridgeInstruction::Unknown => None,
        }
    }
}