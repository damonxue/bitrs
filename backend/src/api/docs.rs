use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// 生成OpenAPI规范
#[derive(OpenApi)]
#[openapi(
    paths(
        // 订单簿API
        crate::api::orderbook::get_orderbook,
        crate::api::orderbook::get_markets,
        crate::api::orderbook::place_order,
        crate::api::orderbook::cancel_order,
        // AMM API
        crate::api::amm::get_pools,
        crate::api::amm::get_pool,
        crate::api::amm::get_swap_quote,
        // 资产API
        crate::api::assets::get_balance,
        crate::api::assets::get_supported_tokens,
        // 交易API
        crate::api::trades::get_recent_trades,
        crate::api::trades::get_user_trades,
    ),
    components(
        schemas(
            // 订单簿模型
            crate::models::OrderBook,
            crate::models::Order,
            crate::models::OrderSide,
            crate::models::Market,
            crate::models::OrderRequest,
            // AMM模型
            crate::models::Pool,
            crate::models::SwapQuoteRequest,
            crate::models::SwapQuoteResponse,
            // 资产模型
            crate::models::Token,
            crate::models::Balance,
            // 交易模型
            crate::models::Trade
        )
    ),
    tags(
        (name = "BitRS DEX API", description = "分布式交易所API")
    ),
    info(
        title = "BitRS DEX API",
        version = env!("CARGO_PKG_VERSION"),
        description = "基于Solana的分布式交易所(DEX)API",
        contact(
            name = "BitRS Team",
            email = "support@bitrs.io",
            url = "https://bitrs.io"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    )
)]
struct ApiDoc;

/// 创建Swagger UI服务
pub fn swagger() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui/{_:.*}")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
}