#![allow(unused)]

use model::ModelController;
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

pub use self::error::{Error, Result};

use axum::{
    extract::{Path, Query},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};

mod error;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    // Intialize the ModelController
    let mc = ModelController::new().await?;

    let routes_all: Router = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", web::routes_tickets::routes(mc.clone()))
        .layer(middleware::map_response(main_response_mapper))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("->> LISTENING on {addr}\n");

    // Create a TcpListener first
    let listener: TcpListener = TcpListener::bind(addr).await.unwrap();

    // Then pass the listener to axum::serve
    axum::serve(listener, routes_all).await.unwrap();

    Ok(())
}

// Region: routes_hello
fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/:name", get(handler_hello2))
}

async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:12} = main_response_mapper", "RES_MAPPER");
    println!();
    res
}

fn routes_static() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// e.g. GET http://localhost:8080/hello?name=Jen
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World!");
    Html(format!("<h1>Hello, {name}</h1>"))
}

async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {name:?}", "HANDLER");

    Html(format!("<h1>Hello, {name}</h1>"))
}
// EndRegion: handler_hello
