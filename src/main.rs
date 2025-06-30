mod errors;
mod handlers;
mod models;
mod utils;

use axum::{
    routing::post,
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(handlers::generate_keypair))
        .route("/token/create", post(handlers::create_token))
        .route("/token/mint", post(handlers::mint_token))
        .route("/message/sign", post(handlers::sign_message))
        .route("/message/verify", post(handlers::verify_message))
        .route("/send/sol", post(handlers::send_sol))
        .route("/send/token", post(handlers::send_token));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
        
    println!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await.unwrap();
}