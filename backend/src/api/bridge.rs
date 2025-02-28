use actix_web::{get, post, web, HttpResponse, Responder};

#[get("/bridge-status")]
async fn bridge_status() -> impl Responder {
    // Logic to fetch and return the status of the cross-chain bridge
    HttpResponse::Ok().json("Bridge status")
}

#[post("/bridge-transfer")]
async fn bridge_transfer() -> impl Responder {
    // Logic to handle cross-chain asset transfer
    HttpResponse::Ok().json("Bridge transfer initiated")
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(bridge_status);
    cfg.service(bridge_transfer);
}
