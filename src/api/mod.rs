use crate::state::AppState;
use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod actor;
pub mod auth;
pub mod extractor;
pub mod guest;
pub mod health;
pub mod invites;
pub mod user;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        auth::login,
        auth::me,
        invites::create_invite,
        invites::accept_invite,
        actor::list_actors,
        actor::get_actor,
        actor::remove_actors,
        user::list_users,
        user::remove_user,
        guest::list_guests,
        guest::remove_guest,
    ),
    components(
        schemas(
            crate::error::ErrorResponse,
            crate::types::Id<()>,
            crate::types::Email,
            crate::types::RawPassword,
            crate::types::HashedPassword,
            crate::domain::role::Role,
            actor::ActorResponse,
            actor::UserActorDetails,
            actor::GuestActorDetails,
            health::HealthResponse,
            auth::LoginRequest,
            auth::UserResponse,
            invites::InviteRequest,
            invites::AcceptInviteRequest,
            user::UserResponse,
            guest::GuestResponse,
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
    .nest("/actors", actor::router())
    .nest("/users", user::router())
    .nest("/guests", guest::router());

  Router::new()
    .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", openapi))
    .nest("/api", api_router)
    .layer(TraceLayer::new_for_http())
    .with_state(state)
}
