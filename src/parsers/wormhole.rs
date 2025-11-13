use crate::error::{Result, StauroXError};
use super::bridge_types::BridgeInstruction;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use tracing::debug;

// Wormhole Token Bridge program ID
const WORMHOLE_TOKEN_BRIDGE: &str = "wormDTUJ6AWPNvk59vGQbDvGJmqbDTdgWgAqcLBCgUb";

// Instruction discriminators (first byte)
const TRANSFER_WRAPPED: u8 = 0x04;
const TRANSFER_NATIVE: u8 = 0x01;

pub fn parse_wormhole_instruction(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<BridgeInstruction> {
    let (account_keys, instructions) = match &tx.transaction.transaction {
        solana_transaction_status::EncodedTransaction::Json(ui_tx) => {
            match &ui_tx.message {
                solana_transaction_status::UiMessage::Raw(msg) => {
                    (&msg.account_keys, &msg.instructions)
                }
                solana_transaction_status::UiMessage::Parsed(_) => {
                    return Err(StauroXError::verification("Parsed message not supported"));
                }
            }
        }
        _ => {
            return Err(StauroXError::verification("Unsupported transaction encoding"));
        }
    };

    // Find Wormhole instruction
    for ix in instructions {
        let program_id = &account_keys[ix.program_id_index as usize];
        
        // Check if this is Wormhole Token Bridge
        if program_id != WORMHOLE_TOKEN_BRIDGE {
            continue;
        }

        // Decode instruction data
        let data = match bs58::decode(&ix.data).into_vec() {
            Ok(d) => d,
            Err(_) => {
                debug!("Failed to decode instruction data");
                continue;
            }
        };

        // Must have at least: discriminator(1) + nonce(4) + amount(8) + fee(8) + address(32) + chain(2) = 55 bytes
        if data.len() < 55 {
            debug!("Instruction data too short: {} bytes", data.len());
            continue;
        }

        // Parse based on discriminator
        let discriminator = data[0];
        
        match discriminator {
            TRANSFER_WRAPPED | TRANSFER_NATIVE => {
                // Parse instruction fields
                let _nonce = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
                
                let amount = u64::from_le_bytes([
                    data[5], data[6], data[7], data[8],
                    data[9], data[10], data[11], data[12],
                ]);
                
                let _fee = u64::from_le_bytes([
                    data[13], data[14], data[15], data[16],
                    data[17], data[18], data[19], data[20],
                ]);
                
                let recipient = data[21..53].to_vec();
                
                let target_chain = u16::from_le_bytes([data[53], data[54]]);

                debug!(
                    "Parsed Wormhole: amount={}, target_chain={}, recipient={}",
                    amount, target_chain, hex::encode(&recipient)
                );

                return Ok(if discriminator == TRANSFER_WRAPPED {
                    BridgeInstruction::TransferWrapped {
                        amount,
                        target_chain,
                        recipient,
                    }
                } else {
                    BridgeInstruction::TransferNative {
                        amount,
                        target_chain,
                        recipient,
                    }
                });
            }
            _ => {
                debug!("Unknown Wormhole instruction discriminator: {}", discriminator);
            }
        }
    }

    Ok(BridgeInstruction::Unknown)
}