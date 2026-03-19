use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{auth, guest, health, invites, permissions, user};
use crate::state::AppState;

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
        permissions::list_permissions,
        guest::list_guests,
    ),
    components(
        schemas(
            crate::error::ErrorResponse,
            crate::types::Id<()>,
            crate::types::Email,
            crate::types::RawPassword,
            crate::types::HashedPassword,
            crate::models::InviteStatus,
            crate::response::UserResponse,
            crate::response::GuestResponse,
            crate::response::HealthResponse,
            crate::response::LoginRequest,
            crate::response::InviteRequest,
            crate::response::InviteResponse,
            crate::response::AcceptInviteRequest,
            crate::models::PermissionDef,
        )
    ),
    tags(
        (name = "cayopay-server", description = "Cayopay Server API")
    )
)]
pub struct ApiDoc;

impl ApiDoc {
  pub fn build(state: &AppState) -> utoipa::openapi::OpenApi {
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
  let openapi = ApiDoc::build(&state);

  let api_router = Router::new()
    .merge(health::router())
    .nest("/auth", auth::router())
    .nest("/invites", invites::router())
    .nest("/users", user::router())
    .nest("/guests", guest::router())
    .nest("/permissions", permissions::router());

  Router::new()
    .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", openapi))
    .nest("/api", api_router)
    .layer(TraceLayer::new_for_http())
    .with_state(state)
}
