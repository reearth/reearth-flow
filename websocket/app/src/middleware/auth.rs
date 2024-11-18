use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware,
};

pub async fn auth_middleware(
    req: Request<Body>,
    next: middleware::Next,
) -> Result<Response<Body>, StatusCode> {
    if req.headers().get("Authorization").is_some() {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
