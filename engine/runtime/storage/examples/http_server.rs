use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
    routing::{get, head},
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let test_data = b"Hello from real HTTP server! This is test GeoJSON data.";

    let app = Router::new()
        .route(
            "/data.geojson",
            head(move || async move {
                println!("Received HEAD request for /data.geojson");
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_LENGTH, test_data.len().to_string())
                    .header(header::LAST_MODIFIED, "Wed, 21 Oct 2015 07:28:00 GMT")
                    .header(header::ETAG, "\"abc123\"")
                    .body(Body::empty())
                    .unwrap()
            }),
        )
        .route(
            "/data.geojson",
            get(move || async move {
                println!("Received GET request for /data.geojson");
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/geo+json")
                    .body(Body::from(test_data.as_slice()))
                    .unwrap()
            }),
        )
        .route(
            "/no-head.json",
            head(|| async move {
                println!("Received HEAD request for /no-head.json (will return 405)");
                Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(Body::from("Method Not Allowed"))
                    .unwrap()
            }),
        )
        .route(
            "/no-head.json",
            get(|| async move {
                println!("Received GET request for /no-head.json");
                let data = b"Server without HEAD support";
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(data.as_slice()))
                    .unwrap()
            }),
        )
        .route(
            "/head-404.json",
            head(|| async move {
                println!("Received HEAD request for /head-404.json (will return 404)");
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap()
            }),
        )
        .route(
            "/head-404.json",
            get(|| async move {
                println!("Received GET request for /head-404.json");
                let data = b"GET succeeds even though HEAD returns 404";
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(data.as_slice()))
                    .unwrap()
            }),
        )
        .route(
            "/head-error.json",
            head(|| async move {
                println!("Received HEAD request for /head-error.json (will return 500)");
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Internal Server Error"))
                    .unwrap()
            }),
        )
        .route(
            "/head-error.json",
            get(|| async move {
                println!("Received GET request for /head-error.json");
                let data = b"GET succeeds even though HEAD returns 500";
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(data.as_slice()))
                    .unwrap()
            }),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("\n🚀 Test HTTP server is running!");
    println!("📍 Address: http://{}", addr);
    println!("\n📚 Available endpoints:");
    println!(
        "  HEAD/GET http://{}/data.geojson       - Full HEAD support with metadata",
        addr
    );
    println!(
        "  HEAD/GET http://{}/no-head.json       - HEAD returns 405 (Method Not Allowed)",
        addr
    );
    println!(
        "  HEAD/GET http://{}/head-404.json      - HEAD returns 404, GET succeeds",
        addr
    );
    println!(
        "  HEAD/GET http://{}/head-error.json    - HEAD returns 500, GET succeeds",
        addr
    );
    println!("\n💡 Test with curl:");
    println!("  curl -I http://{}/data.geojson", addr);
    println!("  curl http://{}/data.geojson", addr);
    println!("\nPress Ctrl+C to stop the server\n");

    axum::serve(listener, app).await.unwrap();
}
