use risc0_zkvm::guest::env;
use sha2::{Sha256, Digest};
use lazy_static::lazy_static;

// Constants for tapleaf header construction
const LEAF_VERSION: u8 = 0xc0; // Tapleaf version
const PADDING_OP: u8 = 0x51; // OP_PUSH_1
const DROP_OP: u8 = 0x75; // OP_DROP
const TAG: &[u8] = b"TapLeaf";

// Compute the hashed tag once at startup
lazy_static! {
    static ref HASHED_TAG: [u8; 32] = {
        let mut hasher = Sha256::new();
        hasher.update(TAG);
        hasher.finalize().into()
    };
}

fn main() {
    let script_len: u32 = env::read();
    let secret_l: [u8; 32] = env::read();
    
    // header = PUSHBYTES_32(1) + L(32) + OP_DROP(1) + padding
    // tapleaf_input = LEAF_VERSION(1) + compact_size(1) + header
    // So header should be 62 bytes to make total input 64 bytes
    let header_len = 62;
    
    let mut header = Vec::with_capacity(header_len as usize);
    
    header.push(0x20); // PUSHBYTES_32
    header.extend_from_slice(&secret_l);
    header.push(DROP_OP);
    
    let padding_len = header_len - 34; // 34 = 1 + 32 + 1
    for _ in 0..padding_len/2 {
        header.push(PADDING_OP);
        header.push(DROP_OP);
    }
    
    assert_eq!(header.len(), header_len as usize, "Header length mismatch");
    
    let total_len = script_len as u64 + header_len as u64;
    
    let mut tapleaf_input = Vec::new();
    tapleaf_input.push(LEAF_VERSION);
    tapleaf_input.push(total_len as u8);
    tapleaf_input.extend_from_slice(&header);
    
    assert_eq!(tapleaf_input.len(), 64, "Tapleaf input must be exactly 64 bytes");
    
    // Implement tagged hash: hash(hash(tag)||hash(tag)||data)
    let mut hasher = Sha256::new();
    hasher.update(&*HASHED_TAG); // First instance of hashed tag
    hasher.update(&*HASHED_TAG); // Second instance of hashed tag
    hasher.update(&tapleaf_input); // The actual data
    let midstate = hasher.finalize();
    
    let midstate_array: [u8; 32] = midstate.into();
    env::commit(&midstate_array);
}
