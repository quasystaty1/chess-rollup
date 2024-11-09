use crate::rollup_app::AppState;
use astria_core::execution::v1::Block;
use astria_core::generated::execution::v1 as execution;
use astria_core::generated::execution::v1::execution_service_server::ExecutionService;
use astria_core::generated::sequencerblock::v1::rollup_data::Value::{Deposit, SequencedData};
use astria_core::primitive::v1::RollupId;
use astria_core::Protobuf;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

pub(crate) struct RollupExecutionService {
    pub app: Arc<RwLock<AppState>>,
}

#[async_trait::async_trait]
impl ExecutionService for RollupExecutionService {
    async fn get_genesis_info(
        self: Arc<Self>,
        request: Request<execution::GetGenesisInfoRequest>,
    ) -> Result<Response<execution::GenesisInfo>, Status> {
        println!("getting genesis info:");
        let _request = request.into_inner();
        let genesis_info = execution::GenesisInfo {
            rollup_id: Some(RollupId::new([69u8; 32]).into_raw()),
            sequencer_genesis_block_height: 2,
            celestia_block_variance: 100,
        };
        println!("genesis_info: {:?}", genesis_info);
        Ok(Response::new(genesis_info))
    }

    async fn get_block(
        self: Arc<Self>,
        request: Request<execution::GetBlockRequest>,
    ) -> Result<Response<execution::Block>, Status> {
        println!("getting block:");
        let state = self.app.read().await;
        let request = request.into_inner();
        match request.identifier {
            Some(identidfier) => match identidfier.identifier {
                Some(id) => match id {
                    execution::block_identifier::Identifier::BlockNumber(height) => {
                        let block: Block = state.get_block(height).unwrap().to_owned();
                        Ok(Response::new(block.into_raw()))
                    }
                    execution::block_identifier::Identifier::BlockHash(_) => {
                        Err(Status::unimplemented("Get Block by hash not implemented"))
                    }
                },
                None => Err(Status::invalid_argument("missing identifier")),
            },
            None => Err(Status::invalid_argument("missing block identifier")),
        }
    }

    async fn batch_get_blocks(
        self: Arc<Self>,
        request: Request<execution::BatchGetBlocksRequest>,
    ) -> Result<Response<execution::BatchGetBlocksResponse>, Status> {
        let request = request.into_inner();
        let state = self.app.read().await;
        let mut blocks = Vec::new();
        for identifier in request.identifiers {
            match identifier.identifier {
                Some(id) => match id {
                    execution::block_identifier::Identifier::BlockNumber(block_number) => {
                        blocks.push(state.get_block(block_number).unwrap().to_owned().into_raw());
                    }
                    execution::block_identifier::Identifier::BlockHash(_) => {
                        return Err(Status::unimplemented("Get Block by hash not implemented"))
                    }
                },
                None => return Err(Status::invalid_argument("missing block identifier")),
            }
        }
        Ok(Response::new(execution::BatchGetBlocksResponse { blocks }))
    }

    async fn execute_block(
        self: Arc<Self>,
        request: Request<execution::ExecuteBlockRequest>,
    ) -> Result<Response<execution::Block>, Status> {
        let request = request.into_inner();
        let timestamp = request.timestamp.unwrap();
        let mut transactions: Vec<Bytes> = Vec::new();
        for rollup_data in request.transactions {
            match rollup_data.value {
                Some(value) => match value {
                    SequencedData(data) => transactions.push(data),
                    Deposit(_) => {}
                },
                None => {}
            };
        }
        let mut state = self.app.write().await;
        let soft_height = state.soft_height;
        let block = state.new_block(
            request.prev_block_hash,
            soft_height + 1,
            transactions,
            timestamp,
        );
        Ok(Response::new(block))
    }

    async fn get_commitment_state(
        self: Arc<Self>,
        _request: Request<execution::GetCommitmentStateRequest>,
    ) -> Result<Response<execution::CommitmentState>, Status> {
        let state = self.app.read().await;
        let soft = state
            .get_block(state.soft_height)
            .and_then(|block| Some(block.to_owned().into_raw()));
        let firm = state
            .get_block(state.firm_height)
            .and_then(|block| Some(block.to_owned().into_raw()));
        let base_celestia_height = state.celestia_height;
        Ok(Response::new(execution::CommitmentState {
            soft,
            firm,
            base_celestia_height,
        }))
    }

    async fn update_commitment_state(
        self: Arc<Self>,
        request: Request<execution::UpdateCommitmentStateRequest>,
    ) -> Result<Response<execution::CommitmentState>, Status> {
        let mut state = self.app.write().await;
        let commitment_state_request = request.into_inner().commitment_state.unwrap();
        let soft_block_request = commitment_state_request.soft.as_ref().unwrap();
        let firm_block_request = commitment_state_request.firm.as_ref().unwrap();
        let soft_request = soft_block_request.number;
        let firm_request = firm_block_request.number;
        let soft_block = state.get_block(soft_request).unwrap().to_owned();
        let firm_block = state.get_block(firm_request).unwrap().to_owned();
        if soft_block.hash().to_owned() != soft_block_request.hash {
            println!(
                "soft block hash does not match: current: {:?},  request: {:?}",
                soft_block.hash().to_owned(),
                soft_block_request.hash
            );
            return Err(Status::invalid_argument("Soft block hash does not match"));
        }
        if firm_block.hash().to_owned() != firm_block_request.hash {
            return Err(Status::invalid_argument("Firm block hash does not match"));
        }
        state.soft_height = soft_request;
        state.firm_height = firm_request;
        let new_commitment_state = execution::CommitmentState {
            soft: Some(soft_block_request.to_owned()),
            firm: Some(firm_block_request.to_owned()),
            base_celestia_height: commitment_state_request.base_celestia_height,
        };

        Ok(Response::new(new_commitment_state))
    }
}
