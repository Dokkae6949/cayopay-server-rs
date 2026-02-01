use crate::app_state::AppState;
use axum::Router;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
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

  let mut openapi = ApiDoc::openapi();
  if let Some(components) = openapi.components.as_mut() {
    components.add_security_scheme(
      "session_cookie",
      SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(
        state.config.session_cookie_name.clone(),
      ))),
    );
  }

  Router::new()
    .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", openapi))
    .nest("/api", api_router)
    .with_state(state)
}
