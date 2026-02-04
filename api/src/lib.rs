use application::AppState;
use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod endpoints;
pub mod error;
pub mod extractor;
pub mod models;

use endpoints::{auth, guest, health, invites, user};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        auth::login,
        auth::me,
        invites::create_invite,
        invites::accept_invite,
        invites::get_invites,
        user::list_users,
        guest::list_guests,
    ),
    components(
        schemas(
            crate::error::ErrorResponse,
            domain::Id<()>,
            domain::Email,
            domain::RawPassword,
            domain::HashedPassword,
            domain::Role,
            domain::InviteStatus,
            models::UserResponse,
            models::GuestResponse,
            models::HealthResponse,
            models::LoginRequest,
            models::InviteRequest,
            models::InviteResponse,
            models::AcceptInviteRequest,
        )
    ),
    tags(
        (name = "cayopay-server", description = "Cayopay Server API")
    )
)]
pub struct ApiDoc;

impl ApiDoc {
  pub fn new(state: &AppState) -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();

    if let Some(components) = openapi.components.as_mut() {
      components.add_security_scheme(
        "session_cookie",
        SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(
          state.config.session_cookie_name.clone(),
        ))),
      );
    }

    openapi
  }
}

pub fn router(state: AppState) -> Router {
  let openapi = ApiDoc::new(&state);

  let api_router = Router::new()
    .merge(health::router())
    .nest("/auth", auth::router())
    .nest("/invites", invites::router())
    .nest("/users", user::router())
    .nest("/guests", guest::router());

  Router::new()
    .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", openapi))
    .nest("/api", api_router)
    .layer(TraceLayer::new_for_http())
    .with_state(state)
}
