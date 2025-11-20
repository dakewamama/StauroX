use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use std::collections::HashMap;
use std::str::FromStr;

const WORMHOLE: &str = "wormDTUJ6AWPNvk59vGQbDvGJmqbDTdgWgAqcLBCgUb";

fn main() {
    println!(" Discovering Wormhole instruction types...\n");
    
    let rpc = RpcClient::new("https://api.mainnet-beta.solana.com");
    
    let sigs = match rpc.get_signatures_for_address(&WORMHOLE.parse().unwrap()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(" Failed: {}", e);
            return;
        }
    };
    
    let mut disc_map: HashMap<u8, Vec<String>> = HashMap::new();
    
    for sig_info in sigs.iter().take(100) {
        if let Ok(sig) = Signature::from_str(&sig_info.signature) {
            if let Ok(tx) = rpc.get_transaction(
                &sig, 
                solana_transaction_status::UiTransactionEncoding::Json
            ) {
                if let solana_transaction_status::EncodedTransaction::Json(ui_tx) = 
                    &tx.transaction.transaction 
                {
                    if let solana_transaction_status::UiMessage::Raw(msg) = &ui_tx.message {
                        for ix in &msg.instructions {
                            let prog = &msg.account_keys[ix.program_id_index as usize];
                            
                            if prog == WORMHOLE {
                                if let Ok(data) = bs58::decode(&ix.data).into_vec() {
                                    if !data.is_empty() {
                                        let disc = data[0];
                                        disc_map
                                            .entry(disc)
                                            .or_insert_with(Vec::new)
                                            .push(sig_info.signature.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!(" Found {} unique instruction types:\n", disc_map.len());
    
    let mut sorted: Vec<_> = disc_map.iter().collect();
    sorted.sort_by_key(|(disc, txs)| (std::cmp::Reverse(txs.len()), **disc));
    
    for (disc, txs) in sorted {
        let name = match *disc {
            0x01 => "TransferNative ( you have this)",
            0x03 => "CompleteTransfer",
            0x04 => "TransferWrapped ( you have this)",
            0x07 => "CompleteTransferNative",
            0x09 => "CreateWrapped",
            _ => "Unknown",
        };
        
        println!("0x{:02x} | {} | {} examples", disc, name, txs.len());
        println!("     Example signature: {}", txs[0]);
        println!();
    }
}