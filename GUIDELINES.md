# Cayopay Server - Coding Guidelines

## General Principles

1. **Keep It Simple**: Write clear, straightforward code that's easy to understand
2. **DRY (Don't Repeat Yourself)**: Extract common patterns into reusable functions
3. **Separation of Concerns**: Respect layer boundaries - no cross-layer violations
4. **Type Safety**: Use the type system to prevent errors at compile time
5. **Fail Fast**: Use Result types and propagate errors with `?`

## Rust Style

### Formatting

- Use `rustfmt` with the project's `.rustfmt.toml` configuration
- 2-space indentation (as configured)
- Run `cargo fmt` before committing

### Naming

- **Types**: `PascalCase` (e.g., `UserService`, `ActorStore`)
- **Functions**: `snake_case` (e.g., `find_by_id`, `list_all`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `OWNER_PERMISSIONS`)
- **Modules**: `snake_case` (e.g., `user_service`, `actor_store`)

### File Organization

```rust
// 1. External imports
use axum::{Json, Router};
use sqlx::PgPool;

// 2. Crate imports
use crate::domain::User;
use crate::error::AppResult;

// 3. Type definitions
pub struct UserService {
  pool: PgPool,
}

// 4. Implementations
impl UserService {
  // Public methods first
  pub fn new(pool: PgPool) -> Self { /* ... */ }
  
  // Private methods last
  fn internal_helper(&self) { /* ... */ }
}

// 5. Tests
#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_something() { /* ... */ }
}
```

## Layer-Specific Guidelines

### API Layer

```rust
// ✅ GOOD: Thin handler, delegates to service
pub async fn list_users(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<UserResponse>>> {
  authz.require(Permission::ReadUserDetails)?;
  let users = state.user_service.list_all().await?;
  Ok(Json(users.into_iter().map(Into::into).collect()))
}

// ❌ BAD: Business logic in handler
pub async fn list_users(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<UserResponse>>> {
  authz.require(Permission::ReadUserDetails)?;
  
  // DON'T: This logic belongs in the service layer
  let users = UserStore::list_all(&state.pool).await?;
  let filtered_users: Vec<User> = users
    .into_iter()
    .filter(|u| u.role != Role::Undefined)
    .collect();
  
  Ok(Json(filtered_users.into_iter().map(Into::into).collect()))
}
```

**API Layer Rules**:
- Always use extractors for auth (`Authn`, `Authz`, `ValidatedJson`)
- Keep handlers to 10-15 lines maximum
- Convert domain entities to DTOs at the handler level
- Use `#[utoipa::path]` for OpenAPI documentation

### Service Layer

```rust
// ✅ GOOD: Focused service method
impl UserService {
  pub async fn list_all(&self) -> AppResult<Vec<User>> {
    UserStore::list_all(&self.pool).await
  }
  
  pub async fn remove_by_id(&self, id: Id<User>) -> AppResult<()> {
    UserStore::remove_by_id(&self.pool, &id).await
  }
}

// ✅ GOOD: Coordinating multiple stores
impl ActorService {
  pub async fn get_actor_by_id(&self, actor_id: &Id<Actor>) -> AppResult<Option<ActorWithDetails>> {
    let actor = ActorStore::find_by_id(&self.pool, actor_id).await?;
    
    match actor {
      Some(actor) => {
        let user = UserStore::find_by_actor_id(&self.pool, actor_id).await?;
        let guest = GuestStore::find_by_actor_id(&self.pool, actor_id).await?;
        Ok(Some(ActorWithDetails { actor, user, guest }))
      }
      None => Ok(None),
    }
  }
}

// ❌ BAD: Pass-through methods that just call stores
impl ActorService {
  // DON'T: This should be in UserService
  pub async fn list_users(&self) -> AppResult<Vec<User>> {
    UserStore::list_all(&self.pool).await
  }
}
```

**Service Layer Rules**:
- Each service owns operations for ONE entity type
- Coordinate between stores when building aggregates
- Use transactions when atomicity is required
- Clone pool when needed (it's cheap - uses Arc internally)

### Store Layer

```rust
// ✅ GOOD: Generic executor pattern
impl UserStore {
  pub async fn save<'c, E>(executor: E, user: &User) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      INSERT INTO users (id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      "#,
      user.id.into_inner(),
      user.actor_id.into_inner(),
      user.email.expose(),
      user.password_hash.expose(),
      user.first_name,
      user.last_name,
      user.role.to_string(),
      user.created_at,
      user.updated_at,
    )
    .execute(executor)
    .await?;

    Ok(())
  }
}

// ❌ BAD: Inconsistent executor type
impl UserStore {
  // DON'T: Use generic executor instead
  pub async fn save(conn: &mut PgConnection, user: User) -> AppResult<()> {
    // ...
  }
}
```

**Store Layer Rules**:
- Use `struct {Name}Store;` (zero-sized type)
- All methods are `async` and take `executor` as first parameter
- Use generic `E: Executor<'c, Database = Postgres>`
- Return `AppResult<T>` for all fallible operations
- Use references for input parameters when possible
- Handle database-specific errors (e.g., unique violations)

### Domain Layer

```rust
// ✅ GOOD: Simple domain entity
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
  pub id: Id<User>,
  pub actor_id: Id<Actor>,
  pub email: Email,
  pub password_hash: HashedPassword,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl User {
  pub fn new(
    actor_id: Id<Actor>,
    email: Email,
    password_hash: HashedPassword,
    first_name: String,
    last_name: String,
    role: Role,
  ) -> Self {
    let now = Utc::now();
    Self {
      id: Id::new(),
      actor_id,
      email,
      password_hash,
      first_name,
      last_name,
      role,
      created_at: now,
      updated_at: now,
    }
  }
}
```

**Domain Layer Rules**:
- Keep entities simple data structures
- Use value objects for validated types (Email, Id, etc.)
- Add constructor methods when initialization logic is needed
- Add domain logic methods when appropriate
- No database queries or HTTP handling

## Error Handling

### Always Use Result Types

```rust
// ✅ GOOD: Returns Result
pub async fn find_user(&self, id: &Id<User>) -> AppResult<Option<User>> {
  UserStore::find_by_id(&self.pool, id).await
}

// ❌ BAD: Uses unwrap/expect
pub async fn find_user(&self, id: &Id<User>) -> Option<User> {
  UserStore::find_by_id(&self.pool, id).await.unwrap()
}
```

### Propagate Errors with ?

```rust
// ✅ GOOD: Clean error propagation
pub async fn register_user(&self, email: Email, password: RawPassword) -> AppResult<User> {
  let hash = password.hash()?;
  let actor = Actor::new();
  ActorStore::save(&self.pool, &actor).await?;
  
  let user = User::new(actor.id, email, hash, /* ... */);
  UserStore::save(&self.pool, &user).await?;
  
  Ok(user)
}

// ❌ BAD: Verbose error handling
pub async fn register_user(&self, email: Email, password: RawPassword) -> AppResult<User> {
  let hash = match password.hash() {
    Ok(h) => h,
    Err(e) => return Err(e),
  };
  // ...
}
```

### Convert Errors at Boundaries

```rust
// ✅ GOOD: Convert database errors to app errors
let result = sqlx::query!(/* ... */)
  .execute(executor)
  .await;

match result {
  Ok(_) => Ok(()),
  Err(e) => Err(match &e {
    sqlx::Error::Database(db_err) => match db_err.kind() {
      sqlx::error::ErrorKind::UniqueViolation => AppError::UserAlreadyExists,
      _ => AppError::Database(e),
    },
    _ => AppError::Database(e),
  }),
}

// ❌ BAD: Expose internal errors
let result = sqlx::query!(/* ... */)
  .execute(executor)
  .await
  .map_err(|e| format!("Database error: {}", e))?; // DON'T: Exposes internals
```

### Log Before Converting

```rust
// ✅ GOOD: Log detailed error before converting
Err(e) => {
  tracing::error!("Failed to send email: {}", e);
  return Err(AppError::InternalServerError);
}

// ❌ BAD: No logging, information lost
Err(_) => return Err(AppError::InternalServerError),
```

## Testing

### Use Result Return Types

```rust
// ✅ GOOD: Returns Result
#[sqlx::test]
async fn test_balance_calculation(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
  let actor = Actor::new();
  ActorStore::save(&pool, &actor).await?;
  
  let balance = TransactionStore::balance_by_wallet_id(&pool, &wallet.id).await?;
  assert_eq!(balance, 0);
  
  Ok(())
}

// ❌ BAD: Uses unwrap
#[sqlx::test]
async fn test_balance_calculation(pool: PgPool) {
  let actor = Actor::new();
  ActorStore::save(&pool, &actor).await.unwrap();
  
  let balance = TransactionStore::balance_by_wallet_id(&pool, &wallet.id).await.unwrap();
  assert_eq!(balance, 0);
}
```

### Test One Thing

```rust
// ✅ GOOD: Focused test
#[test]
fn test_owner_has_configure_permission() {
  assert!(Role::Owner.has_permission(Permission::ConfigureSettings));
}

#[test]
fn test_admin_lacks_configure_permission() {
  assert!(!Role::Admin.has_permission(Permission::ConfigureSettings));
}

// ❌ BAD: Testing multiple things
#[test]
fn test_all_permissions() {
  assert!(Role::Owner.has_permission(Permission::ConfigureSettings));
  assert!(!Role::Admin.has_permission(Permission::ConfigureSettings));
  assert!(Role::Admin.has_permission(Permission::InviteUser));
  // ...many more assertions
}
```

## Performance

### Use References

```rust
// ✅ GOOD: Uses references
pub async fn find_by_id<'c, E>(executor: E, id: &Id<User>) -> AppResult<Option<User>>

// ❌ BAD: Takes ownership unnecessarily
pub async fn find_by_id<'c, E>(executor: E, id: Id<User>) -> AppResult<Option<User>>
```

### Avoid Allocations in Hot Paths

```rust
// ✅ GOOD: Returns slice reference
pub fn permissions(&self) -> &'static [Permission] {
  match self {
    Role::Owner => Self::OWNER_PERMISSIONS,
    Role::Admin => Self::ADMIN_PERMISSIONS,
    Role::Undefined => &[],
  }
}

// ❌ BAD: Allocates new Vec every call
pub fn permissions(&self) -> Vec<Permission> {
  match self {
    Role::Owner => vec![Permission::ConfigureSettings, /* ... */],
    Role::Admin => vec![Permission::InviteUser, /* ... */],
    Role::Undefined => vec![],
  }
}
```

### Clone Pool Freely

```rust
// ✅ GOOD: Clone pool (it's Arc<T> internally)
impl UserService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

let user_service = UserService::new(pool.clone());
let guest_service = GuestService::new(pool.clone());
```

## Documentation

### Document Public APIs

```rust
/// Creates a new user account and returns the created user.
///
/// This function:
/// 1. Hashes the password
/// 2. Creates an actor
/// 3. Creates the user with the specified role
///
/// # Errors
///
/// Returns `AppError::UserAlreadyExists` if the email is already registered.
/// Returns `AppError::PasswordHash` if password hashing fails.
pub async fn register(
  &self,
  email: Email,
  password: RawPassword,
  first_name: String,
  last_name: String,
  role: Role,
) -> AppResult<User> {
  // ...
}
```

### Use OpenAPI Documentation

```rust
#[utoipa::path(
  get,
  context_path = "/api/users",
  path = "/",
  responses(
    (status = StatusCode::OK, description = "List of all users", body = Vec<UserResponse>),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
    (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
  )
)]
pub async fn list_users(/* ... */) -> AppResult<Json<Vec<UserResponse>>> {
  // ...
}
```

## Code Review Checklist

Before submitting code:

- [ ] Run `cargo fmt` to format code
- [ ] Run `cargo clippy` to check for common mistakes
- [ ] Ensure no `.unwrap()` or `.expect()` in production code
- [ ] Verify layer boundaries are respected
- [ ] Check that error handling is consistent
- [ ] Confirm tests pass
- [ ] Update documentation if needed
- [ ] Remove any TODO comments or create issues for them
- [ ] Ensure commit messages are clear and descriptive

## Common Mistakes to Avoid

### ❌ Direct Store Access from Endpoints

```rust
// DON'T
pub async fn list_users(State(state): State<AppState>) -> AppResult<Json<Vec<UserResponse>>> {
  let users = UserStore::list_all(&state.pool).await?;  // Wrong layer
  Ok(Json(users.into_iter().map(Into::into).collect()))
}

// DO
pub async fn list_users(State(state): State<AppState>) -> AppResult<Json<Vec<UserResponse>>> {
  let users = state.user_service.list_all().await?;  // Correct
  Ok(Json(users.into_iter().map(Into::into).collect()))
}
```

### ❌ Business Logic in Stores

```rust
// DON'T
impl UserStore {
  pub async fn find_active_users<'c, E>(executor: E) -> AppResult<Vec<User>> {
    // Business logic: "active" is a business concept
    let users = sqlx::query_as!(/* ... */)
      .fetch_all(executor)
      .await?;
    
    Ok(users.into_iter().filter(|u| u.is_active()).collect())
  }
}

// DO: Move business logic to service
impl UserService {
  pub async fn find_active_users(&self) -> AppResult<Vec<User>> {
    let users = UserStore::list_all(&self.pool).await?;
    Ok(users.into_iter().filter(|u| u.is_active()).collect())
  }
}
```

### ❌ Using Domain Types in Config

```rust
// DON'T
pub struct Config {
  pub smtp_username: Email,
  pub smtp_password: RawPassword,
}

// DO
pub struct Config {
  pub smtp_username: String,
  pub smtp_password: String,
}
```

### ❌ Panicking in Library Code

```rust
// DON'T
pub async fn get_user(&self, id: &Id<User>) -> User {
  UserStore::find_by_id(&self.pool, id)
    .await
    .unwrap()  // Will panic if user not found!
}

// DO
pub async fn get_user(&self, id: &Id<User>) -> AppResult<Option<User>> {
  UserStore::find_by_id(&self.pool, id).await
}
```

## Questions?

If you're unsure about a pattern or approach:

1. Check existing code for similar patterns
2. Refer to ARCHITECTURE.md for layer responsibilities
3. Ask in PR review
4. Update these guidelines with the resolution

## Continuous Improvement

These guidelines evolve with the project. When you find a better pattern or approach:

1. Discuss with the team
2. Update this document
3. Refactor existing code when beneficial
