// use solana_client::rpc_client::RpcClient;
// use solana_sdk::signature::Signature;
// use solana_transaction_status::UiTransactionEncoding;
// use std::str::FromStr;

// fn main() {
//     let sig = "5zvidmQcZXe3YVYWbBwPC8Se78XWSqev1sFFhH6RnoCDHeX7bxAL3Y8FAxkLumrAv5tCquvn5fx5rfv4MkaE4xaj";
    
//     let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
//     let signature = Signature::from_str(sig).unwrap();
    
//     println!("Analyzing: {}\n", signature);
    
//     let config = solana_client::rpc_config::RpcTransactionConfig {
//         encoding: Some(UiTransactionEncoding::Json),
//         commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
//         max_supported_transaction_version: Some(0),
//     };
    
//     match client.get_transaction_with_config(&signature, config) {
//         Ok(tx) => {
//             if let solana_transaction_status::EncodedTransaction::Json(ui_tx) = &tx.transaction.transaction {
//                 let (account_keys, instructions) = match &ui_tx.message {
//                     solana_transaction_status::UiMessage::Raw(msg) => {
//                         (&msg.account_keys, &msg.instructions)
//                     }
//                     solana_transaction_status::UiMessage::Parsed(_) => {
//                         println!("Parsed message not supported");
//                         return;
//                     }
//                 };
                
//                 println!("ACCOUNT KEYS:");
//                 for (i, key) in account_keys.iter().enumerate() {
//                     println!("  [{}] {}", i, key);
//                 }
                
//                 println!("\nINSTRUCTIONS:");
//                 for (i, ix) in instructions.iter().enumerate() {
//                     println!("\nInstruction {}:", i);
//                     let program_key = &account_keys[ix.program_id_index as usize];
//                     println!("  Program: {}", program_key);
//                     println!("  Data (base58): {}", ix.data);
                    
//                     if let Ok(decoded) = bs58::decode(&ix.data).into_vec() {
//                         println!("  Hex: {}", hex::encode(&decoded));
//                         println!("  Length: {} bytes", decoded.len());
                        
//                         if decoded.len() >= 8 {
//                             println!("\n  PARSED:");
//                             println!("  Discriminator: {:?}", &decoded[0..8]);
                            
//                             let mut offset = 8;
//                             if decoded.len() >= offset + 4 {
//                                 let nonce = u32::from_le_bytes([
//                                     decoded[offset], decoded[offset+1], 
//                                     decoded[offset+2], decoded[offset+3]
//                                 ]);
//                                 println!("  Nonce: {}", nonce);
//                                 offset += 4;
//                             }
                            
//                             if decoded.len() >= offset + 8 {
//                                 let amount = u64::from_le_bytes([
//                                     decoded[offset], decoded[offset+1], decoded[offset+2], decoded[offset+3],
//                                     decoded[offset+4], decoded[offset+5], decoded[offset+6], decoded[offset+7],
//                                 ]);
//                                 println!("  Amount: {}", amount);
//                                 offset += 8;
//                             }
                            
//                             if decoded.len() >= offset + 8 {
//                                 let fee = u64::from_le_bytes([
//                                     decoded[offset], decoded[offset+1], decoded[offset+2], decoded[offset+3],
//                                     decoded[offset+4], decoded[offset+5], decoded[offset+6], decoded[offset+7],
//                                 ]);
//                                 println!("  Fee: {}", fee);
//                                 offset += 8;
//                             }
                            
//                             if decoded.len() >= offset + 32 {
//                                 println!("  Target Address: {}", hex::encode(&decoded[offset..offset+32]));
//                                 offset += 32;
//                             }
                            
//                             if decoded.len() >= offset + 2 {
//                                 let chain = u16::from_le_bytes([decoded[offset], decoded[offset+1]]);
//                                 println!("  Target Chain: {}", chain);
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         Err(e) => {
//             eprintln!("Error: {}", e);
//         }
//     }
// 




use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use std::env;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin analyze_wormhole <signature>");
        return;
    }
    
    let sig = Signature::from_str(&args[1]).expect("Invalid signature");
    let rpc = RpcClient::new("https://api.mainnet-beta.solana.com");
    
    println!(" Analyzing: {}\n", args[1]);
    
    let tx = rpc.get_transaction(
        &sig, 
        solana_transaction_status::UiTransactionEncoding::Json
    ).expect("Failed to get transaction");
    
    if let solana_transaction_status::EncodedTransaction::Json(ui_tx) = 
        &tx.transaction.transaction 
    {
        if let solana_transaction_status::UiMessage::Raw(msg) = &ui_tx.message {
            for ix in &msg.instructions {
                let prog = &msg.account_keys[ix.program_id_index as usize];
                
                if prog.contains("worm") {
                    let data = bs58::decode(&ix.data).into_vec().unwrap();
                    
                    println!("Discriminator: 0x{:02x}", data[0]);
                    println!("Length: {} bytes\n", data.len());
                    
                    println!("Hex bytes:");
                    for chunk in data.chunks(16) {
                        print!("  ");
                        for byte in chunk {
                            print!("{:02x} ", byte);
                        }
                        println!();
                    }
                    
                    println!("\nNow figure out what each byte means!");
                }
            }
        }
    }
}