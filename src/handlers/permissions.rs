use axum::{routing::get, Json, Router};

use crate::{
  models::{PermissionDef, PERMISSIONS},
  state::AppState,
};

/// List all available permission definitions.
///
/// Returns the static catalogue of every permission the system recognises,
/// including its human-readable description and the resource type it targets.
/// This endpoint is intended for UIs that need to render permission selectors.
#[utoipa::path(
    get,
    path = "/api/permissions",
    responses(
        (status = StatusCode::OK, description = "List of permission definitions", body = Vec<PermissionDef>),
    )
)]
pub async fn list_permissions() -> Json<&'static [PermissionDef]> {
  Json(PERMISSIONS)
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_permissions))
}
