use axum::{
  extract::{Path, State},
  http::StatusCode,
  routing::{delete, get},
  Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
  api::extractor::Authz,
  domain::{actor::ActorWithDetails, Actor, Guest, Permission, Role, User},
  error::{AppError, AppResult, ErrorResponse},
  state::AppState,
  types::{Email, Id},
};

#[derive(Serialize, ToSchema)]
pub struct UserActorDetails {
  pub user_id: Id<User>,
  pub email: Email,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
}

#[derive(Serialize, ToSchema)]
pub struct GuestActorDetails {
  pub guest_id: Id<Guest>,
}

#[derive(Serialize, ToSchema)]
pub struct ActorResponse {
  pub id: Id<Actor>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<UserActorDetails>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub guest: Option<GuestActorDetails>,
}

impl From<ActorWithDetails> for ActorResponse {
  fn from(details: ActorWithDetails) -> Self {
    let user = details.user.map(|u| UserActorDetails {
      user_id: u.id,
      email: u.email,
      first_name: u.first_name,
      last_name: u.last_name,
      role: u.role,
    });

    let guest = details.guest.map(|g| GuestActorDetails { guest_id: g.id });

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
  authz.require(Permission::ReadActorDetails)?;

  let can_read_user = authz.require(Permission::ReadUserDetails).is_ok();
  let can_read_guest = authz.require(Permission::ReadGuestDetails).is_ok();

  let actors = state.actor_service.list_actors().await?;
  let response: Vec<ActorResponse> = actors
    .into_iter()
    .map(|details| {
      let mut resp = ActorResponse::from(details);
      if !can_read_user {
        resp.user = None;
      }
      if !can_read_guest {
        resp.guest = None;
      }
      resp
    })
    .collect();

  Ok(Json(response))
}

#[utoipa::path(
    get,
    context_path = "/api/actors",
    path = "/{actor_id}",
    responses(
        (status = StatusCode::OK, description = "Actor details", body = ActorResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
        (status = StatusCode::NOT_FOUND, description = "Actor not found", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn get_actor(
  State(state): State<AppState>,
  authz: Authz,
  Path(actor_id): Path<Id<Actor>>,
) -> AppResult<Json<ActorResponse>> {
  authz.require(Permission::ReadActorDetails)?;

  let can_read_user = authz.require(Permission::ReadUserDetails).is_ok();
  let can_read_guest = authz.require(Permission::ReadGuestDetails).is_ok();

  let actor = state
    .actor_service
    .get_actor_by_id(&actor_id)
    .await?
    .map(|details| {
      let mut resp = ActorResponse::from(details);
      if !can_read_user {
        resp.user = None;
      }
      if !can_read_guest {
        resp.guest = None;
      }
      resp
    })
    .ok_or_else(|| AppError::NotFound)?;

  Ok(Json(actor))
}

// Remove an actor by ID passed as a path parameter
#[utoipa::path(
    delete,
    context_path = "/api/actors",
    path = "/{actor_id}",
    responses(
        (status = StatusCode::NO_CONTENT, description = "Actor removed successfully"),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
        (status = StatusCode::NOT_FOUND, description = "Actor not found", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn remove_actors(
  State(state): State<AppState>,
  authz: Authz,
  Path(actor_id): Path<Id<Actor>>,
) -> AppResult<StatusCode> {
  authz.require(Permission::RemoveActor)?;

  state.actor_service.remove_by_id(actor_id).await?;

  Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", get(list_actors))
    .route("/{actor_id}", get(get_actor))
    .route("/{actor_id}", delete(remove_actors))
}
