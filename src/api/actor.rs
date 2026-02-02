use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
  api::extractor::Authz,
  domain::{actor::ActorWithDetails, Actor, Guest, Permission, Role, User},
  error::{AppResult, ErrorResponse},
  state::AppState,
  stores::ActorStore,
  types::{Email, Id},
};

#[derive(Serialize, ToSchema)]
pub struct UserActorResponse {
  pub id: Id<User>,
  pub email: Email,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
}

#[derive(Serialize, ToSchema)]
pub struct GuestActorResponse {
  pub id: Id<Guest>,
}

#[derive(Serialize, ToSchema)]
pub struct ActorResponse {
  pub id: Id<Actor>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<UserActorResponse>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub guest: Option<GuestActorResponse>,
}

impl From<ActorWithDetails> for ActorResponse {
  fn from(details: ActorWithDetails) -> Self {
    let user = details.user.map(|u| UserActorResponse {
      id: u.id,
      email: u.email,
      first_name: u.first_name,
      last_name: u.last_name,
      role: u.role,
    });

    let guest = details.guest.map(|g| GuestActorResponse { id: g.id });

    Self {
      id: details.actor.id,
      created_at: details.actor.created_at,
      updated_at: details.actor.updated_at,
      user,
      guest,
    }
  }
}

/// List all actors
#[utoipa::path(
    get,
    context_path = "/api/actors",
    path = "/",
    responses(
        (status = StatusCode::OK, description = "List of all actors", body = Vec<ActorResponse>),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn list_actors(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<ActorResponse>>> {
  authz.require(Permission::ViewAllActors)?;

  let actors = ActorStore::list_all_detailed(&state.pool).await?;
  let response: Vec<ActorResponse> = actors.into_iter().map(Into::into).collect();

  Ok(Json(response))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_actors))
}
