use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::api::extractor::Authz;
use crate::domain::actor::ActorDetailResponse;
use crate::domain::Permission;
use crate::error::AppResult;
use crate::state::AppState;
use crate::stores::ActorStore;

/// List all actors
#[utoipa::path(
    get,
    context_path = "/api/actors",
    path = "/",
    responses(
        (status = StatusCode::OK, description = "List of all actors", body = Vec<ActorWithDetails>),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn list_actors(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<ActorDetailResponse>>> {
  authz.require(Permission::ViewAllActors)?;

  let actors = ActorStore::list_all_detailed(&state.pool).await?;
  Ok(Json(actors))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_actors))
}
