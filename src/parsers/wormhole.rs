use crate::error::{Result, StauroXError};
use super::bridge_types::BridgeInstruction;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use tracing::debug;

const WORMHOLE_TOKEN_BRIDGE: &str = "wormDTUJ6AWPNvk59vGQbDvGJmqbDTdgWgAqcLBCgUb";

// Instruction discriminators
const TRANSFER_NATIVE: u8 = 0x01;
const ATTEST_TOKEN: u8 = 0x02;
const COMPLETE_TRANSFER: u8 = 0x03;
const TRANSFER_WRAPPED: u8 = 0x04;
const TRANSFER_TOKENS_WITH_PAYLOAD: u8 = 0x05;
const COMPLETE_TRANSFER_NATIVE: u8 = 0x07;
const CREATE_WRAPPED: u8 = 0x09;
const COMPLETE_WRAPPED: u8 = 0x0a;
const COMPLETE_TRANSFER_WITH_PAYLOAD: u8 = 0x0d;

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

    for ix in instructions {
        let program_id = &account_keys[ix.program_id_index as usize];
        
        if program_id != WORMHOLE_TOKEN_BRIDGE {
            continue;
        }

        let data = match bs58::decode(&ix.data).into_vec() {
            Ok(d) => d,
            Err(_) => {
                debug!("Failed to decode instruction data");
                continue;
            }
        };

        if data.is_empty() {
            continue;
        }

        let discriminator = data[0];
        
        return match discriminator {
            TRANSFER_NATIVE | TRANSFER_WRAPPED | TRANSFER_TOKENS_WITH_PAYLOAD => {
                parse_transfer_instruction(discriminator, &data)
            }
            ATTEST_TOKEN => {
                parse_attest_token()
            }
            COMPLETE_TRANSFER | COMPLETE_TRANSFER_NATIVE => {
                parse_complete_transfer(discriminator)
            }
            CREATE_WRAPPED | COMPLETE_WRAPPED => {
                parse_wrapped_token_instruction(discriminator)
            }
            COMPLETE_TRANSFER_WITH_PAYLOAD => {
                parse_complete_transfer_with_payload(&data)
            }
            _ => {
                debug!("Unknown Wormhole instruction: 0x{:02x}", discriminator);
                Ok(BridgeInstruction::Unknown)
            }
        };
    }

    Ok(BridgeInstruction::Unknown)
}

fn parse_transfer_instruction(discriminator: u8, data: &[u8]) -> Result<BridgeInstruction> {
    // TransferNative (0x01), TransferWrapped (0x04), TransferTokensWithPayload (0x05)
    // All share the same layout: disc(1) + nonce(4) + amount(8) + fee(8) + recipient(32) + chain(2) = 55 bytes
    
    if data.len() < 55 {
        debug!("Transfer instruction too short: {} bytes", data.len());
        return Ok(BridgeInstruction::Unknown);
    }

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
        "Parsed Wormhole transfer (0x{:02x}): amount={}, chain={}, recipient={}",
        discriminator, amount, target_chain, hex::encode(&recipient)
    );

    Ok(match discriminator {
        TRANSFER_WRAPPED => BridgeInstruction::TransferWrapped {
            amount,
            target_chain,
            recipient,
        },
        TRANSFER_NATIVE => BridgeInstruction::TransferNative {
            amount,
            target_chain,
            recipient,
        },
        TRANSFER_TOKENS_WITH_PAYLOAD => BridgeInstruction::TransferWithPayload {
            amount,
            target_chain,
        },
        _ => BridgeInstruction::Unknown,
    })
}

fn parse_attest_token() -> Result<BridgeInstruction> {
    // AttestToken (0x02): Register a new token for bridging
    // Instruction data is ONLY the discriminator (1 byte)
    // Token attestation data is stored in accounts
    
    debug!("Parsed AttestToken instruction");
    
    Ok(BridgeInstruction::AttestToken)
}

fn parse_complete_transfer(discriminator: u8) -> Result<BridgeInstruction> {
    // CompleteTransfer (0x03), CompleteTransferNative (0x07)
    // Receive tokens on Solana from another chain
    // Instruction data is ONLY the discriminator (1 byte)
    // VAA (Verified Action Approval) data is stored in accounts
    
    debug!(
        "Parsed CompleteTransfer: discriminator=0x{:02x}, is_native={}",
        discriminator, 
        discriminator == COMPLETE_TRANSFER_NATIVE
    );
    
    Ok(BridgeInstruction::CompleteTransfer {
        vaa_hash: vec![], // Empty - VAA is in accounts
        is_native: discriminator == COMPLETE_TRANSFER_NATIVE,
    })
}

fn parse_wrapped_token_instruction(discriminator: u8) -> Result<BridgeInstruction> {
    // CreateWrapped (0x09), CompleteWrapped (0x0a)
    // Token creation/completion operations
    // Instruction data is minimal (just discriminator)
    
    debug!(
        "Wrapped token instruction: discriminator=0x{:02x}",
        discriminator
    );
    
    Ok(BridgeInstruction::WrappedTokenOperation {
        operation_type: if discriminator == CREATE_WRAPPED {
            "CreateWrapped".to_string()
        } else {
            "CompleteWrapped".to_string()
        },
    })
}

fn parse_complete_transfer_with_payload(data: &[u8]) -> Result<BridgeInstruction> {
    // CompleteTransferWithPayload (0x0d): Receive transfer with metadata
    // Similar to CompleteTransfer but includes payload
    
    debug!(
        "Parsed CompleteTransferWithPayload: length={}",
        data.len()
    );
    
    Ok(BridgeInstruction::CompleteTransferWithPayload)
}