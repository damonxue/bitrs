use actix::{Actor, StreamHandler, Message, Handler, ActorContext};
use actix_web_actors::ws;
use std::sync::Arc;
use serde::Serialize;
use tokio::sync::broadcast;
use crate::analytics::{SystemMetrics, PoolMetrics};

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct MetricsUpdate(pub SystemMetrics);

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct PoolUpdate(pub PoolMetrics);

pub struct MetricsWsSession {
    metrics_tx: broadcast::Sender<MetricsUpdate>,
    pool_tx: broadcast::Sender<PoolUpdate>,
}

impl Actor for MetricsWsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Subscribe to metrics updates
        let mut metrics_rx = self.metrics_tx.subscribe();
        let addr = ctx.address();
        
        // Handle metrics updates
        actix::spawn(async move {
            while let Ok(update) = metrics_rx.recv().await {
                addr.do_send(update);
            }
        });

        // Handle pool updates
        let mut pool_rx = self.pool_tx.subscribe();
        let addr = ctx.address();
        
        actix::spawn(async move {
            while let Ok(update) = pool_rx.recv().await {
                addr.do_send(update);
            }
        });
    }
}

impl Handler<MetricsUpdate> for MetricsWsSession {
    type Result = ();

    fn handle(&mut self, msg: MetricsUpdate, ctx: &mut Self::Context) {
        if let Ok(json) = serde_json::to_string(&msg.0) {
            ctx.text(json);
        }
    }
}

impl Handler<PoolUpdate> for MetricsWsSession {
    type Result = ();

    fn handle(&mut self, msg: PoolUpdate, ctx: &mut Self::Context) {
        if let Ok(json) = serde_json::to_string(&msg.0) {
            ctx.text(json);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MetricsWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // Handle incoming messages if needed
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct WsServer {
    metrics_tx: broadcast::Sender<MetricsUpdate>,
    pool_tx: broadcast::Sender<PoolUpdate>,
}

impl WsServer {
    pub fn new() -> Self {
        let (metrics_tx, _) = broadcast::channel(100);
        let (pool_tx, _) = broadcast::channel(100);
        Self {
            metrics_tx,
            pool_tx,
        }
    }

    pub fn broadcast_metrics(&self, metrics: SystemMetrics) {
        let _ = self.metrics_tx.send(MetricsUpdate(metrics));
    }

    pub fn broadcast_pool_update(&self, pool: PoolMetrics) {
        let _ = self.pool_tx.send(PoolUpdate(pool));
    }

    pub fn create_session(&self) -> MetricsWsSession {
        MetricsWsSession {
            metrics_tx: self.metrics_tx.clone(),
            pool_tx: self.pool_tx.clone(),
        }
    }
}