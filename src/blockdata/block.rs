// block.rs
//
// Block structures module
//
//


pub struct BlockHeader {
    version: u32,
    prev_hash: Hash,
    merkle_root: Hash,
    timestamp: u32,
    bits: u32,
    nonce: u32,
    txn_count: VariableInteger
}