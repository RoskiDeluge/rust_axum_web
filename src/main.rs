#![allow(unused)]

use model::ModelController;
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

pub use self::error::{Error, Result};

use axum::{
    extract::{Path, Query},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Json, Router,
};

mod ctx;
mod error;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    // Intialize the ModelController
    let mc: ModelController = ModelController::new().await?;

    let routes_apis = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all: Router = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
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
    // Generate a single UUID for the entire request/response cycle
    let uuid = Uuid::new_v4();

    // -- Get the eventual response error
    let service_error: Option<&Error> = res.extensions().get::<Error>();

    // Log the server log line with the UUID regardless of whether there was an error
    println!("   ->> server log line - {uuid} - Error: {service_error:?}");

    // Return early with a custom error response if there's an error
    if let Some(err) = service_error {
        let (status_code, client_error) = err.client_status_and_error();

        let client_error_body = json!({
            "error": {
                "type": client_error.as_ref(),
                "req_uuid": uuid.to_string(),
            }
        });

        println!("   ->> client_error_body: {client_error_body}");
        return (status_code, Json(client_error_body)).into_response();
    }

    println!();
    res
}

// async fn main_response_mapper(res: Response) -> Response {
//     println!("->> {:12} = main_response_mapper", "RES_MAPPER");
//     let uuid = Uuid::new_v4();

//     // -- Get the eventual response error
//     let service_error: Option<&Error> = res.extensions().get::<Error>();
//     let client_status_error: Option<(axum::http::StatusCode, error::ClientError)> =
//         service_error.map(|se: &Error| se.client_status_and_error());

//     // If client error, build the new response.
//     // let error_response: Option<&(axum::http::StatusCode, error::ClientError)> = client_status_error
//     //     .as_ref()
//     //     .map(|(status_code, client_error)| {
//     //         let client_error_body = json!({
//     //             "error": {
//     //                 "type": client_error.as_ref(),
//     //                 "req_uuid": uuid.to_string(),
//     //             }
//     //         });
//     //         println!("   ->> client_error_body: {client_error_body}");

//     //         // Build the new response from the client_error_body
//     //         (*status_code, Json(client_error_body)).into_response()
//     //     });

//     println!();
//     res
// }

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
