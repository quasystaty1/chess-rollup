use astria_core::generated::execution::v1 as execution;
use astria_core::generated::execution::v1::execution_service_server::ExecutionService;
use astria_core::generated::primitive::v1::*;
use astria_core::generated::sequencerblock::v1::rollup_data::Value::{Deposit, SequencedData};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct RollupExecutionService {}

#[async_trait::async_trait]
impl ExecutionService for RollupExecutionService {
    async fn get_genesis_info(
        self: Arc<Self>,
        request: Request<execution::GetGenesisInfoRequest>,
    ) -> Result<Response<execution::GenesisInfo>, Status> {
        println!("getting genesis info:");
        let _request = request.into_inner();
        let genesis_info = execution::GenesisInfo {
            rollup_id: Some(RollupId {
                inner: bytes::Bytes::from("chess"),
            }),
            sequencer_genesis_block_height: 2,
            celestia_block_variance: 2,
        };
        Ok(Response::new(genesis_info))
    }

    async fn get_block(
        self: Arc<Self>,
        request: Request<execution::GetBlockRequest>,
    ) -> Result<Response<execution::Block>, Status> {
        println!("getting block:");
        let request = request.into_inner();
        match request.identifier {
            Some(identidfier) => match identidfier.identifier {
                Some(id) => match id {
                    execution::block_identifier::Identifier::BlockNumber(number) => {
                        Ok(Response::new(execution::Block::default()))
                    }
                    execution::block_identifier::Identifier::BlockHash(hash) => {
                        Ok(Response::new(execution::Block::default()))
                    }
                },
                None => Ok(Response::new(execution::Block::default())),
            },
            None => Ok(Response::new(execution::Block::default())),
        }
    }

    async fn batch_get_blocks(
        self: Arc<Self>,
        request: Request<execution::BatchGetBlocksRequest>,
    ) -> Result<Response<execution::BatchGetBlocksResponse>, Status> {
        let request = request.into_inner();
        let mut blocks = Vec::new();
        for identifier in request.identifiers {
            match identifier.identifier {
                Some(id) => match id {
                    execution::block_identifier::Identifier::BlockNumber(_) => {
                        blocks.push(execution::Block::default());
                    }
                    execution::block_identifier::Identifier::BlockHash(_) => {
                        blocks.push(execution::Block::default());
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
        for rollup_data in request.transactions {
            let transaction = match rollup_data.value {
                Some(value) => {
                    match value {
                        SequencedData(data) => {
                            // Placeholder value
                            todo!()
                        }
                        Deposit(_) => {}
                    }
                }
                None => todo!(),
            };
        }
        Ok(Response::new(execution::Block::default()))
    }

    async fn get_commitment_state(
        self: Arc<Self>,
        request: Request<execution::GetCommitmentStateRequest>,
    ) -> Result<Response<execution::CommitmentState>, Status> {
        // TODO: get current soft
        // TODO: get current firm
        // TODO: get current celestia height
        // TODO: construct commitment state
        Ok(Response::new(execution::CommitmentState {
            soft: Some(execution::Block::default()),
            firm: Some(execution::Block::default()),
            base_celestia_height: 2,
        }))
    }

    async fn update_commitment_state(
        self: Arc<Self>,
        request: Request<execution::UpdateCommitmentStateRequest>,
    ) -> Result<Response<execution::CommitmentState>, Status> {
        // TODO: compate commitment soft and firm hash with rollup one
        Ok(Response::new(execution::CommitmentState::default()))
    }
}
