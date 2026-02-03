use crate::{
  api::{extractor::Authz, models::ActorResponse},
  domain::{Actor, Permission},
  error::{AppError, AppResult, ErrorResponse},
  state::AppState,
  types::Id,
};
use axum::{
  extract::{Path, State},
  http::StatusCode,
  routing::{delete, get},
  Json, Router,
};

fn filter_actor_response(mut response: ActorResponse, authz: &Authz) -> ActorResponse {
  if authz.require(Permission::ReadUserDetails).is_err() {
    response.user = None;
  }
  if authz.require(Permission::ReadGuestDetails).is_err() {
    response.guest = None;
  }
  response
}

/// List all actors.
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

  let actors = state.actor_service.list_actors().await?;
  let response: Vec<ActorResponse> = actors
    .into_iter()
    .map(|details| filter_actor_response(ActorResponse::from(details), &authz))
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

  let actor = state
    .actor_service
    .get_actor_by_id(&actor_id)
    .await?
    .map(|details| filter_actor_response(ActorResponse::from(details), &authz))
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
