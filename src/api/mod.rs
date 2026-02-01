use crate::app_state::AppState;
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod auth;
pub mod extractor;
pub mod health;
pub mod invites;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        auth::login,
        auth::me,
        invites::create_invite,
    ),
    components(
        schemas(
            crate::error::ErrorResponse,
            crate::types::Id<crate::domain::User>,
            crate::types::Email,
            crate::types::RawPassword,
            crate::types::HashedPassword,
            health::HealthResponse,
            auth::LoginRequest,
            auth::UserResponse,
            invites::InviteRequest,
            invites::InviteResponse,
        )
    ),
    tags(
        (name = "cayopay-server", description = "Cayopay Server API")
    )
)]
pub struct ApiDoc;

pub fn router(state: AppState) -> Router {
  let api_router = Router::new()
    .merge(health::router())
    .nest("/auth", auth::router())
    .nest("/invites", invites::router());

  Router::new()
    .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi()))
    .nest("/api", api_router)
    .with_state(state)
}
