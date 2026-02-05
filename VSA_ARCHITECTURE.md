# Vertical Slice Architecture (VSA)

## Philosophy

**Share types, not behavior. Each feature is completely self-contained.**

## Structure

Each feature is ONE file containing:
- HTTP handler
- Database queries (inline, no abstraction)
- DTOs (request/response models)
- Feature-specific errors with `IntoResponse`
- All business logic

## Example: login.rs

```rust
//! Login Feature - Everything in one place

// DTOs
pub struct Request { email: String, password: String }
pub struct ResponseDto { id: String, email: String, ... }

// Feature-specific errors
pub enum Error {
    InvalidCredentials,
    Database(sqlx::Error),
    ...
}
impl IntoResponse for Error { ... }

// Database queries (inline)
async fn find_user(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as("SELECT ... FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

// Handler
pub async fn handle(
    State(pool): State<PgPool>,
    Json(req): Json<Request>,
) -> Result<Json<ResponseDto>, Error> {
    let user = find_user(&pool, &req.email).await?.ok_or(Error::InvalidCredentials)?;
    // verify password, create session, return response
    ...
}

pub fn router() -> Router<PgPool> {
    Router::new().route("/login", post(handle))
}
```

## Features

| Feature | File | Business Capability |
|---------|------|---------------------|
| `login.rs` | 150 LOC | User authentication & session creation |
| `me.rs` | 120 LOC | Get current authenticated user |
| `send_invite.rs` | 180 LOC | Send user invitation via email |
| `accept_invite.rs` | 140 LOC | Accept invite & register new user |
| `list_invites.rs` | 130 LOC | View all invitations (admin) |
| `list_users.rs` | 120 LOC | View all users (admin) |
| `list_guests.rs` | 120 LOC | View all guests (admin) |

## Shared Components

**Only types, no behavior:**
- `domain` crate: `Email`, `Role`, `Permission`, etc.
- `config.rs`: Minimal configuration loading

## Benefits

1. **Easy to understand**: Everything for a feature in one place
2. **Easy to change**: Modify one feature without affecting others
3. **Easy to test**: Each feature can be tested independently
4. **No over-abstraction**: Direct SQL queries, no repository pattern
5. **Fast to develop**: No need to navigate multiple layers

## Anti-Patterns Avoided

❌ Shared stores/repositories layer
❌ Shared service layer
❌ Separate models for each layer (DB model, domain model, DTO)
❌ Abstract email service with method per email type
❌ Shared error types

✅ Direct DB queries in features
✅ Feature-specific errors
✅ Generic `Mail` object
✅ One model per concern (no layer duplication)
