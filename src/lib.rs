pub mod features;
pub mod shared;

use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use shared::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Health
        shared::health::health_check,
        // Auth
        features::auth::handler::login,
        features::auth::handler::me,
        // Onboarding
        features::onboarding::handler::send_invite,
        features::onboarding::handler::accept_invite,
        features::onboarding::handler::get_invites,
        // User Management
        features::user_management::handler::list_users,
        // Guest Management
        features::guest_management::handler::list_guests,
    ),
    components(
        schemas(
            shared::error::ErrorResponse,
            shared::health::HealthResponse,
            domain::Id<()>,
            domain::Email,
            domain::RawPassword,
            domain::HashedPassword,
            domain::Role,
            domain::InviteStatus,
            features::auth::models::LoginRequest,
            features::auth::models::UserResponse,
            features::onboarding::models::InviteRequest,
            features::onboarding::models::InviteResponse,
            features::onboarding::models::AcceptInviteRequest,
            features::user_management::models::UserDetailResponse,
            features::guest_management::models::GuestDetailResponse,
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

/// Create the main application router with all feature slices
pub fn router(state: AppState) -> Router {
  let openapi = ApiDoc::new(&state);

  let api_router = Router::new()
    .merge(shared::health_router())
    .nest("/auth", features::auth::router())
    .nest("/invites", features::onboarding::router())
    .nest("/users", features::user_management::router())
    .nest("/guests", features::guest_management::router());

  Router::new()
    .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", openapi))
    .nest("/api", api_router)
    .layer(TraceLayer::new_for_http())
    .with_state(state)
}
