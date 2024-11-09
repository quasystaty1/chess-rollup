use crate::config::Config;
use crate::execution_service::{self, RollupExecutionService};
use crate::rollup_app::AppState;
use astria_core::generated::execution::v1::execution_service_server::{
    ExecutionService, ExecutionServiceServer,
};
use astria_core::generated::execution::v1::{ExecuteBlockRequest, GetCommitmentStateRequest};
use color_eyre::eyre;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct Chess;
use std::net::SocketAddr;
use tonic::transport::Server;
impl Chess {
    pub async fn run_until_stopped(config: Config) -> eyre::Result<()> {
        let addr: SocketAddr = "0.0.0.0:50051".parse()?;
        let mut app = AppState::new();
        let mutex_app = Arc::new(RwLock::new(app));
        let execution_service = RollupExecutionService { app: mutex_app };

        println!("ExecutionServiceServer listening on {}", addr);
        Server::builder()
            .add_service(ExecutionServiceServer::new(execution_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}
