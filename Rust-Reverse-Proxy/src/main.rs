//! Reverse proxy listening in "localhost:4000" will proxy all requests to "localhost:3000"
//! endpoint.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -p reverse-proxy
//! ```

use axum::{
    extract::Extension,
    http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
use hyper::{client::HttpConnector, Body};
use std::{convert::TryFrom, net::SocketAddr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type Client = hyper::client::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "reverse_proxy=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // tokio::spawn(server());

    let client = Client::new();

    let app = Router::new()
        .route("/", get(handler))
        .route("/anything", get(handler))
        .layer(Extension(client));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    tracing::debug!("reverse proxy listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(
    Extension(client): Extension<Client>,
    // NOTE: Make sure to put the request extractor last because once the request
    // is extracted, extensions can't be extracted anymore.
    mut req: Request<Body>,
) -> Response<Body> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("http://httpbin.org{}", path_query);
    tracing::debug!("uri is {}", uri);
    tracing::debug!("headers are {:?}", &req.headers());

    *req.uri_mut() = Uri::try_from(uri).unwrap();

    let res = client.request(req).await.unwrap();
    tracing::debug!("response are {:?}", res);
    res
}

// async fn server() {
//     let app = Router::new()
//         .route("/", get(|| async { "Hello, world!" }))
//         .route("/hello", get(|| async { "Hello, hello!" }));

//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//     tracing::debug!("server listening on {}", addr);
//     axum::Server::bind(&addr)
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
// }
