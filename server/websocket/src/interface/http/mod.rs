pub mod controllers;
pub mod dto;
pub mod middleware;
pub mod routes;
pub mod validators;

// Re-export commonly used items following DDD structure
pub use controllers::DocumentController;
pub use dto::*;
pub use middleware::*;
pub use routes::document_routes;
pub use validators::*;
