//! CayoPay Server - True Vertical Slice Architecture
//!
//! Each feature is self-contained with:
//! - Handlers
//! - Database queries (inline, no stores)
//! - DTOs
//! - Feature-specific errors
//!
//! No over-abstraction, most direct approach

pub mod config;
pub mod features;

use axum::Router;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

use crate::features::send_invite::SmtpConfig;

#[derive(OpenApi)]
#[openapi(
    paths(
        features::health::handle,
        features::login::handle,
        features::me::handle,
        features::send_invite::handle,
        features::accept_invite::handle,
        features::list_invites::handle,
        features::list_users::handle,
        features::list_guests::handle,
    ),
    components(
        schemas(
            domain::Role,
            features::health::ResponseDto,
            features::login::Request,
            features::login::ResponseDto,
            features::me::ResponseDto,
            features::send_invite::Request,
            features::accept_invite::Request,
            features::list_invites::InviteResponse,
            features::list_users::UserResponse,
            features::list_guests::GuestResponse,
        )
    ),
    tags(
        (name = "cayopay", description = "CayoPay Server API")
    )
)]
struct ApiDoc;

impl ApiDoc {
    fn with_security() -> utoipa::openapi::OpenApi {
        let mut openapi = ApiDoc::openapi();
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "session_cookie",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("cayopay_session"))),
            );
        }
        openapi
    }
}

pub fn app(pool: PgPool, smtp: SmtpConfig) -> Router {
    let openapi = ApiDoc::with_security();
    
    // Create state tuple for send_invite
    let invite_state = (pool.clone(), smtp.clone());
    
    // Each feature router gets appropriate state
    let api_router = Router::new()
        .merge(features::health::router())
        .nest("/auth", 
            Router::new()
                .merge(features::login::router())
                .merge(features::me::router())
                .with_state(pool.clone())
        )
        .nest("/invites",
            Router::new()
                .merge(features::list_invites::router().with_state(pool.clone()))
                .merge(features::accept_invite::router().with_state(pool.clone()))
                .merge(features::send_invite::router().with_state(invite_state))
        )
        .nest("/users", features::list_users::router().with_state(pool.clone()))
        .nest("/guests", features::list_guests::router().with_state(pool));
    
    Router::new()
        .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", openapi))
        .nest("/api", api_router)
        .layer(TraceLayer::new_for_http())
}
