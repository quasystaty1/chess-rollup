use crate::config::Config;
use crate::execution_service::RollupExecutionService;
use astria_core::generated::execution::v1::execution_service_server::ExecutionServiceServer;

use color_eyre::eyre;
pub struct chess;
use std::net::SocketAddr;
use tonic::transport::Server;

impl chess {
    pub async fn run_until_stopped(config: Config) -> eyre::Result<()> {
        let addr: SocketAddr = "0.0.0.0:50051".parse()?;
        let execution_service = RollupExecutionService {};

        println!("ExecutionServiceServer listening on {}", addr);

        Server::builder()
            .add_service(ExecutionServiceServer::new(execution_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}
