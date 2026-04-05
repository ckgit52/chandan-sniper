use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::pubkey::Pubkey;
use bs58;
use std::str::FromStr;

pub struct Wallet {
    pub keypair: Keypair,
    pub pubkey: Pubkey,
}

impl Wallet {
    pub fn from_private_key(private_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let decoded = bs58::decode(private_key).into_vec()?;
        let keypair = Keypair::from_bytes(&decoded)?;
        let pubkey = keypair.pubkey();
        
        Ok(Self { keypair, pubkey })
    }
    
    pub fn pubkey_string(&self) -> String {
        self.pubkey.to_string()
    }
}
