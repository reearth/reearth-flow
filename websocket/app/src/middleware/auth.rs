use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware,
};

pub async fn auth_middleware(
    req: Request<Body>,
    next: middleware::Next,
) -> Result<Response<Body>, StatusCode> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        // Extract the token
        let auth_header = auth_header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let _token = &auth_header[7..];

        // TODO: Implement token validation
        // Example: validate_token(token).await?;

        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
