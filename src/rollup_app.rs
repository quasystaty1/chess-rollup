use astria_core::{execution::v1::Block, Protobuf};
use astria_sequencer_client::{
    tendermint::{serializers::timestamp, time::ParseTimestamp},
    tendermint_proto::google::protobuf::Timestamp,
};
use bytes::Bytes;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AppState {
    pub blocks: HashMap<u32, Block>, // store blocks indexed by block number
    pub soft_height: u32,            // current soft height
    pub firm_height: u32,
    pub celestia_height: u64,
}

impl AppState {
    pub fn new() -> Self {
        let block = astria_core::generated::execution::v1::Block {
            number: 0,
            hash: Bytes::from_static(&[69_u8; 32]),
            parent_block_hash: Bytes::new(),
            timestamp: Some(pbjson_types::Timestamp {
                seconds: 0,
                nanos: 0,
            }),
        };
        let mut blocks: HashMap<u32, Block> = HashMap::new();
        blocks.insert(0, Block::try_from_raw(block).unwrap());
        AppState {
            blocks: blocks,
            soft_height: 0,
            firm_height: 0,
            celestia_height: 2,
        }
    }

    // Add a new block (it must be greater than both current heights)
    pub fn add_block(&mut self, block: Block) {
        let block_height = block.number();
        if block_height <= self.soft_height {
            println!(
                "Error: New block number must be greater than soft height {}.",
                self.soft_height,
            );
            return;
        }

        // Insert the new block into the HashMap
        self.blocks.insert(block.number(), block);

        // Update the soft height to the latest block number
        self.soft_height = block_height;
    }

    // Set the firm height (firm height cannot be larger than soft height)
    pub fn set_firm_height(&mut self, new_firm_height: u32) {
        if new_firm_height > self.soft_height {
            println!(
                "Error: Firm height cannot be greater than soft height. Current soft height: {}",
                self.soft_height
            );
            return;
        }

        self.firm_height = new_firm_height;
    }

    pub fn get_parent_hash(&self, block_number: u32) -> Bytes {
        let block = self.blocks.get(&block_number).unwrap();
        block.parent_block_hash().clone()
    }

    // Retrieve a block by its number
    pub fn get_block(&self, block_number: u32) -> Option<&Block> {
        self.blocks.get(&block_number)
    }

    pub fn new_block(
        &mut self,
        parent_hash: Bytes,
        height: u32,
        tx_to_process: Vec<Bytes>,
        timestamp: pbjson_types::Timestamp,
    ) -> astria_core::generated::execution::v1::Block {
        let block = astria_core::generated::execution::v1::Block {
            number: height,
            hash: parent_hash.clone(),
            parent_block_hash: parent_hash,
            timestamp: Some(timestamp),
        };
        self.add_block(Block::try_from_raw_ref(&block).unwrap());
        println!("New block: {:?}", block);
        block
    }
}
