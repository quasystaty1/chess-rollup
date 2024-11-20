use crate::config::Config;
use crate::execution_service::{self, RollupExecutionService};
use crate::game::{self, GameManager};
use crate::rollup_app::AppState;
use astria_core::generated::composer::v1::grpc_collector_service_client::GrpcCollectorServiceClient;
use astria_core::generated::composer::v1::{
    SubmitRollupTransactionRequest, SubmitRollupTransactionResponse,
};
use astria_core::generated::execution::v1::execution_service_server::{
    ExecutionService, ExecutionServiceServer,
};
use astria_core::generated::primitive::v1::RollupId;
use bytes::Bytes;
use color_eyre::eyre;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
pub struct Chess;
use crate::game::Transaction;
use std::net::SocketAddr;
use tonic::transport::Server;
impl Chess {
    pub async fn run_until_stopped(mut config: Config) -> eyre::Result<()> {
        let addr: SocketAddr = config.grpc_addr.parse()?;
        let composer_addr = config.composer_addr;
        println!("composer address: {}", composer_addr);
        let mut composer_client = GrpcCollectorServiceClient::connect(composer_addr.clone())
            .await
            .unwrap();
        let game_manager = GameManager::new(Bytes::from_static(&[69_u8; 32]));
        // `POST /create_game` endpoint to create a new game
        let create_game = warp::path!("create_game" / u32)
            .and(warp::post())
            .and(with_composer(composer_client.clone()))
            .and_then(handle_create_game);
        // `GET /game_status/{game_id}` endpoint to get game status
        let game_status = warp::path!("game_status" / u32)
            .and(warp::get())
            .and(with_game_manager(game_manager.clone()))
            .and_then(handle_get_game_status);

        let routes = create_game;

        println!("Rest server listening on {}", 3030);
        // Spawn the server in a separate async task so it doesn't block the main program
        tokio::spawn(async move {
            warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
        });

        let app = AppState::new();
        let mut game_manager = game::GameManager::new(Bytes::from_static(&[69_u8; 32]));
        let mutex_game_manager = Arc::new(RwLock::new(game_manager));
        let mutex_app = Arc::new(RwLock::new(app));
        let execution_service = RollupExecutionService {
            app: mutex_app,
            game_manager: mutex_game_manager,
        };

        println!("ExecutionServiceServer listening on {}", addr);
        Server::builder()
            .add_service(ExecutionServiceServer::new(execution_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}

// Helper function to pass `GameManager` as a filter to endpoints
fn with_composer(
    composer_client: GrpcCollectorServiceClient<tonic::transport::channel::Channel>,
) -> impl Filter<
    Extract = (GrpcCollectorServiceClient<tonic::transport::channel::Channel>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || composer_client.clone())
}

// Helper function to pass `GameManager` as a filter to endpoints
fn with_game_manager(
    game_manager: game::GameManager,
) -> impl Filter<Extract = (game::GameManager,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || game_manager.clone())
}

// Handler for `POST /create_game/{game_id}`
async fn handle_create_game(
    game_id: u32,
    mut composer_client: GrpcCollectorServiceClient<tonic::transport::channel::Channel>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let transaction = Transaction::StartGame { game_id: game_id };
    println!("encoding transaction: {:?}", transaction);
    let encoded_transaction = transaction.encode();
    println!(
        "submitting transaction to sequencer... encoded transaction {:?}",
        encoded_transaction
    );
    let composer_response = composer_client
        .submit_rollup_transaction(SubmitRollupTransactionRequest {
            rollup_id: Some(RollupId {
                inner: Bytes::from_static(&[69_u8; 32]),
            }),
            data: encoded_transaction,
        })
        .await
        .unwrap();
    Ok(warp::reply::json(&format!(
        "Game {} transaction submitted to sequencer",
        game_id
    )))
}

// Handler for `GET /game_status/{game_id}`
async fn handle_get_game_status(
    game_id: u32,
    game_manager: game::GameManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    match game_manager.game_status(game_id) {
        Ok(status) => {
            let response = format!("{:?}", status);
            Ok(warp::reply::json(&response))
        }
        Err(_) => Err(warp::reject::not_found()),
    }
}
