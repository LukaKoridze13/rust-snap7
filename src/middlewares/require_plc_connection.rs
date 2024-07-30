use axum::{
    response::Response,
    middleware:: Next,
    extract::Request,
};

pub async fn require_plc_connection(
    request: Request,
    next: Next,
) -> Response {
    println!("** Running Middleware");
    let response = next.run(request).await;
    response
}