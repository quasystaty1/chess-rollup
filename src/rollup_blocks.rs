use astria_core::generated::execution::v1 as execution;
use bytes::Bytes;
use sha2::{Digest, Sha256};
pub(crate) struct Block {
    pub height: u32,
    pub hash: Bytes,
    pub parent_hash: Bytes,
    pub transactions: Vec<Bytes>,
}

pub(crate) fn new_block(parent_hash: Bytes, txs: Vec<Bytes>, height: u32) -> Block {
    Block {
        height: height,
        hash: hash_txs(txs.clone()),
        parent_hash: parent_hash,
        transactions: txs,
    }
}

fn hash_txs(txs: Vec<Bytes>) -> Bytes {
    let mut hasher = Sha256::new();
    // Iterate over each `Bytes` in the vector and update the hasher
    for chunk in txs {
        hasher.update(chunk);
    }
    let result = hasher.finalize();
    Bytes::copy_from_slice(result.as_slice())
}

impl Block {
    fn to_astria_pb(&self) -> execution::Block {
        let mut block = execution::Block {
            number: self.height,
            parent_block_hash: self.parent_hash.clone(),
            timestamp: todo!(),
            hash: self.hash,
        };
    }
}
