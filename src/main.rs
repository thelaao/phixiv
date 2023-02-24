use std::{env, sync::Arc};

use axum::{body::Body, extract::Host, routing::get, Router};
use http::Request;
use phixiv::{
    embed::embed_router, phixiv::phixiv_router, pixiv_redirect, proxy::proxy_router, PhixivState,
};
use tokio::sync::RwLock;
use tower::ServiceExt;

use tower_http::{normalize_path::NormalizePathLayer, trace::{TraceLayer, DefaultOnEos, DefaultOnRequest}};
use tracing::Level;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::fmt().with_file(true).init();

    let state = Arc::new(RwLock::new(PhixivState::new().await.unwrap()));

    let phixiv = phixiv_router(state.clone());
    let embed = embed_router();
    let proxy = proxy_router(state.clone());

    let app = Router::new()
        .route(
            "/*path",
            get(|Host(hostname): Host, request: Request<Body>| async move {
                match hostname.split_once(".") {
                    Some(("i", _)) => {
                        tracing::info!("Hostname: i.*");
                        proxy.oneshot(request).await
                    }
                    Some(("e", _)) => {
                        tracing::info!("Hostname: e.*");
                        embed.oneshot(request).await
                    }
                    _ => {
                        tracing::info!("Hostname: *.*");
                        phixiv.oneshot(request).await
                    }
                }
            }),
        )
        .fallback(pixiv_redirect)
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("[::]:{}", env::var("PORT").unwrap_or("3000".to_owned()))
        .parse()
        .unwrap();

    tracing::info!("Listening on: {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
