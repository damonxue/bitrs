use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_web_actors::ws;
use std::sync::Arc;
use solana_sdk::pubkey::Pubkey;

mod api;
mod tokenomics;
mod reward_handler;
mod buyback;
mod mev;
mod errors;
mod analytics;
mod middleware;
mod rate_limiter;
mod config;
mod models;
mod solana;
mod ws;

use crate::{
    tokenomics::TokenEconomics,
    reward_handler::RewardHandler,
    buyback::BuybackManager,
    analytics::Analytics,
    middleware::MonitoringMiddleware,
    config::Config,
    ws::WsServer,
};

pub struct AppState {
    token_economics: Arc<TokenEconomics>,
    buyback_manager: Arc<BuybackManager>,
    analytics: Arc<Analytics>,
    config: Arc<Config>,
    ws_server: Arc<WsServer>,
}

async fn ws_handler(
    req: web::HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> Result<web::HttpResponse, actix_web::Error> {
    let ws_session = data.ws_server.create_session();
    ws::start(ws_session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Load configuration
    let config = Arc::new(Config::load().expect("Failed to load configuration"));
    log::info!("Configuration loaded successfully");

    // Initialize WebSocket server
    let ws_server = Arc::new(WsServer::new());

    // Initialize components
    let treasury = Pubkey::new_unique(); // Replace with actual treasury pubkey
    let dex_token_mint = Pubkey::new_unique(); // Replace with actual token mint

    // Initialize TokenEconomics
    let token_economics = Arc::new(TokenEconomics::new(
        treasury,
        dex_token_mint,
    ));

    // Initialize Analytics with WebSocket support
    let analytics = Arc::new(Analytics::new(
        token_economics.clone(),
        ws_server.clone(),
    ));
    analytics.start_monitoring().await;

    // Initialize RewardHandler
    let reward_handler = RewardHandler::new(token_economics.clone());
    reward_handler.start_reward_distribution_task().await;

    // Initialize BuybackManager
    let buyback_manager = Arc::new(BuybackManager::new(
        token_economics.clone(),
        config.tokenomics.min_buyback_amount,
        0.01, // 1% max slippage
    ));
    buyback_manager.start_buyback_service().await;

    // Create shared application state
    let app_state = web::Data::new(AppState {
        token_economics: token_economics.clone(),
        buyback_manager: buyback_manager.clone(),
        analytics: analytics.clone(),
        config: config.clone(),
        ws_server: ws_server.clone(),
    });

    // Initialize monitoring middleware
    let monitoring = MonitoringMiddleware::new(analytics.clone());

    // Start HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(monitoring.clone())
            .app_data(app_state.clone())
            .service(web::resource("/ws").route(web::get().to(ws_handler)))
            .configure(api::init_routes)
    })
    .bind(format!("{}:{}", config.api.host, config.api.port))?
    .run();

    log::info!("Server started at http://{}:{}", config.api.host, config.api.port);
    server.await
}