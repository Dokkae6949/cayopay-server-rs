# Cayopay Server - Architecture Documentation

## Overview

Cayopay Server is a REST API server built with Rust, using Axum web framework and PostgreSQL database. The application follows a clean layered architecture with clear separation of concerns.

## Architecture Layers

```
┌─────────────────────────────────────────────────────┐
│                   API Layer                          │
│  - Endpoints (HTTP handlers)                         │
│  - Models (DTOs/Request/Response types)              │
│  - Extractors (Authentication/Authorization)         │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│                Service Layer                         │
│  - Business logic orchestration                      │
│  - Transaction coordination                          │
│  - Service classes (UserService, ActorService, etc.) │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│                Store Layer                           │
│  - Database access (Repository pattern)              │
│  - SQL queries with sqlx                             │
│  - Store classes (UserStore, ActorStore, etc.)       │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│                Domain Layer                          │
│  - Domain entities (User, Actor, Guest, etc.)        │
│  - Value objects (Id<T>, Email, HashedPassword)      │
│  - Business rules and validations                    │
└─────────────────────────────────────────────────────┘
```

## Layer Responsibilities

### API Layer (`src/api/`)

**Purpose**: Handle HTTP requests and responses

**Responsibilities**:
- Parse and validate incoming HTTP requests
- Route requests to appropriate handlers
- Authenticate and authorize requests
- Serialize responses to JSON
- Handle HTTP-specific concerns (status codes, headers)

**Key Components**:
- **Endpoints** (`endpoints/`): HTTP handler functions
- **Models** (`models/`): Request/Response DTOs
- **Extractors** (`extractor/`): Custom Axum extractors for auth

**Rules**:
- ✅ DO validate request input
- ✅ DO use extractors for authentication/authorization
- ✅ DO keep handlers thin - delegate to services
- ❌ DON'T put business logic in handlers
- ❌ DON'T access stores directly
- ❌ DON'T handle database transactions

### Service Layer (`src/services/`)

**Purpose**: Implement business logic and orchestrate operations

**Responsibilities**:
- Coordinate multiple store operations
- Implement business rules and workflows
- Handle transactions when needed
- Provide clean APIs for the API layer

**Key Services**:
- **ActorService**: Manage actors and aggregate actor details
- **UserService**: User-specific operations
- **GuestService**: Guest-specific operations
- **AuthService**: Authentication and registration
- **InviteService**: Invitation workflow
- **SessionService**: Session management
- **EmailService**: Email sending

**Rules**:
- ✅ DO contain all business logic
- ✅ DO coordinate between multiple stores
- ✅ DO handle transactions
- ✅ DO return domain entities
- ❌ DON'T handle HTTP concerns
- ❌ DON'T parse request bodies
- ❌ DON'T serialize responses

### Store Layer (`src/stores/`)

**Purpose**: Provide database access through the Repository pattern

**Responsibilities**:
- Execute SQL queries
- Map database rows to domain entities
- Provide CRUD operations
- Handle database-specific errors

**Key Stores**:
- **UserStore**: User database operations
- **ActorStore**: Actor database operations
- **GuestStore**: Guest database operations
- **InviteStore**: Invitation database operations
- **SessionStore**: Session database operations
- **WalletStore**: Wallet database operations
- **TransactionStore**: Transaction database operations

**Rules**:
- ✅ DO use generic `Executor<'c, Database = Postgres>` pattern
- ✅ DO accept references to entities when possible
- ✅ DO return `AppResult<T>` for error handling
- ✅ DO use `sqlx::query!` and `sqlx::query_as!` macros
- ❌ DON'T contain business logic
- ❌ DON'T coordinate between stores
- ❌ DON'T expose database-specific types in signatures

**Standard Store Pattern**:
```rust
pub struct UserStore;

impl UserStore {
  pub async fn save<'c, E>(executor: E, user: &User) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(/* ... */)
      .execute(executor)
      .await?;
    Ok(())
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &Id<User>) -> AppResult<Option<User>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let user = sqlx::query_as!(/* ... */)
      .fetch_optional(executor)
      .await?;
    Ok(user)
  }
}
```

### Domain Layer (`src/domain/`)

**Purpose**: Define core business entities and rules

**Responsibilities**:
- Define domain entities (structs)
- Implement domain-specific logic
- Enforce business invariants
- Provide type safety

**Key Entities**:
- **User**: Registered user account
- **Guest**: Guest user account
- **Actor**: Base entity for users and guests
- **Invite**: Invitation to join
- **Session**: User session
- **Wallet**: User wallet
- **Transaction**: Financial transaction
- **Role**: User role (Owner, Admin)
- **Permission**: Granular permissions

**Rules**:
- ✅ DO contain domain logic
- ✅ DO enforce invariants
- ✅ DO be serializable/deserializable
- ❌ DON'T depend on infrastructure
- ❌ DON'T contain database queries
- ❌ DON'T contain HTTP handling

### Types Layer (`src/types/`)

**Purpose**: Provide reusable value objects

**Key Types**:
- **Id<T>**: Type-safe UUID wrapper with phantom type
- **Email**: Email address with validation
- **RawPassword**: Plain text password (never persisted)
- **HashedPassword**: Hashed password for storage

**Rules**:
- ✅ DO provide type safety
- ✅ DO validate on construction
- ✅ DO be immutable when possible
- ❌ DON'T contain business logic

## Error Handling

All application errors use the `AppError` enum defined in `src/error.rs`:

```rust
pub enum AppError {
  Database(sqlx::Error),
  Validation(validator::ValidationErrors),
  Authentication,
  Authorization,
  NotFound,
  UserAlreadyExists,
  InviteAlreadySent,
  InviteExpired,
  BadRequest(String),
  InternalServerError,
  PasswordHash(argon2::password_hash::Error),
}

pub type AppResult<T> = Result<T, AppError>;
```

**Error Handling Rules**:
- ✅ DO use `AppResult<T>` for all fallible operations
- ✅ DO convert errors at layer boundaries
- ✅ DO log sensitive errors before converting to generic ones
- ✅ DO use `?` operator for error propagation
- ❌ DON'T use `.unwrap()` or `.expect()` in production code
- ❌ DON'T expose internal errors to API consumers
- ❌ DON'T panic in library code

## Configuration

Configuration is loaded from environment variables or `.env` file:

```rust
pub struct Config {
  pub host: String,
  pub port: u16,
  pub database_url: String,
  pub smtp_host: String,
  pub smtp_username: String,  // Plain string
  pub smtp_password: String,  // Plain string
  // ...
}
```

**Configuration Rules**:
- ✅ DO use primitive types (String, u16, i64, bool)
- ✅ DO convert to domain types at service boundaries
- ✅ DO provide sensible defaults
- ❌ DON'T use domain value objects in Config
- ❌ DON'T store secrets in code

## Database Access

### Generic Executor Pattern

All store methods use the generic executor pattern for flexibility:

```rust
pub async fn save<'c, E>(executor: E, entity: &Entity) -> AppResult<()>
where
  E: Executor<'c, Database = Postgres>,
{
  sqlx::query!(/* ... */)
    .execute(executor)
    .await?;
  Ok(())
}
```

This allows the same method to work with:
- `&PgPool` - Connection pool (most common)
- `&mut PgConnection` - Single connection
- `&mut Transaction` - Database transaction

### Transactions

When multiple operations must succeed or fail together:

```rust
let mut tx = pool.begin().await?;

ActorStore::save(&mut tx, &actor).await?;
UserStore::save(&mut tx, &user).await?;

tx.commit().await?;
```

## Authentication & Authorization

### Authentication (Authn)

The `Authn` extractor verifies the user's identity:

```rust
pub async fn me(Authn(user): Authn) -> AppResult<Json<UserResponse>> {
  Ok(Json(user.into()))
}
```

### Authorization (Authz)

The `Authz` extractor checks permissions:

```rust
pub async fn delete_user(authz: Authz, /* ... */) -> AppResult<StatusCode> {
  authz.require(Permission::RemoveUser)?;
  // ... perform operation
}
```

### Permission System

Permissions are role-based:

```rust
impl Role {
  pub fn has_permission(&self, perm: Permission) -> bool {
    self.permissions().contains(&perm)
  }
}
```

- **Owner**: All permissions including ConfigureSettings
- **Admin**: Most permissions except ConfigureSettings
- **Undefined**: No permissions

## Testing

### Unit Tests

Place tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_role_permissions() {
    assert!(Role::Owner.has_permission(Permission::ConfigureSettings));
  }
}
```

### Integration Tests

Database tests use `#[sqlx::test]`:

```rust
#[sqlx::test]
async fn test_balance_calculation(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
  // ... test implementation
  Ok(())
}
```

**Testing Rules**:
- ✅ DO use `Result<()>` return types instead of unwrap
- ✅ DO use `?` operator for error propagation
- ✅ DO clean up test data when needed
- ❌ DON'T use `.unwrap()` or `.expect()` in tests
- ❌ DON'T leave TODO comments in tests

## Best Practices

### Code Organization

- Keep files under 200 lines when possible
- Use modules to group related functionality
- Extract helper functions to reduce duplication
- Separate concerns by layer

### Naming Conventions

- **Services**: `{Entity}Service` (e.g., `UserService`)
- **Stores**: `{Entity}Store` (e.g., `UserStore`)
- **Models**: `{Entity}Response`, `{Action}Request`
- **Methods**: Verb-first (e.g., `create_user`, `find_by_id`)

### Error Messages

- Be specific in error messages for internal logging
- Be generic in error messages for API responses
- Log errors with context before converting

### Performance

- Use const arrays for static data (e.g., role permissions)
- Avoid allocations in hot paths
- Use references when ownership isn't needed
- Batch database queries when possible

## Common Patterns

### Creating a New Entity

1. Define entity in `src/domain/`
2. Create store in `src/stores/`
3. Create service in `src/services/`
4. Add endpoints in `src/api/endpoints/`
5. Add models in `src/api/models/`
6. Update OpenAPI documentation

### Adding a New Permission

1. Add to `Permission` enum in `src/domain/permission.rs`
2. Update role permission arrays in `src/domain/role.rs`
3. Use in endpoints with `authz.require(Permission::NewPerm)?`

### Adding a New Endpoint

1. Create handler function in appropriate `endpoints/` file
2. Add `#[utoipa::path]` documentation
3. Register route in router function
4. Add to `ApiDoc` struct in `src/api/mod.rs`

## Further Reading

- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Utoipa (OpenAPI) Documentation](https://docs.rs/utoipa/)
