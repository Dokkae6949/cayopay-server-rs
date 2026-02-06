# Code Duplication Strategy in VSA

## Current State (8 Features)

### Duplication Overview

```
┌─────────────────────────────────────────────────────────────┐
│ Current Duplication: ~50 lines across 8 features            │
│ Average: 6 lines per feature                                 │
│ Status: ✅ HEALTHY (within acceptable range)                │
└─────────────────────────────────────────────────────────────┘

Repeated Patterns:
├── Auth Session Check (5 features)      ~8 lines each = 40 lines
├── Error Handling (7 features)          ~15 lines each = 105 lines
└── Permission Checks (4 features)       ~3 lines each = 12 lines
                                         Total: ~157 lines duplicated
```

### Why This Is OK

**1. Feature Independence**
```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  Feature A   │  │  Feature B   │  │  Feature C   │
│              │  │              │  │              │
│ ┌──────────┐ │  │ ┌──────────┐ │  │ ┌──────────┐ │
│ │   Auth   │ │  │ │   Auth   │ │  │ │   Auth   │ │
│ │  Logic   │ │  │ │  Logic   │ │  │ │  Logic   │ │
│ └──────────┘ │  │ └──────────┘ │  │ └──────────┘ │
│              │  │              │  │              │
│  Can change  │  │ Independently│  │  No cascade  │
│  freely      │  │  evolve      │  │  effects     │
└──────────────┘  └──────────────┘  └──────────────┘
```

**2. Change Isolation**
- Modify Feature A's auth → Only Feature A changes
- No cascading refactors
- Easy rollbacks

**3. Simplicity Wins**
```
Duplication (Current)          vs.    Abstraction (Premature)
├── Easy to understand                ├── Hidden dependencies
├── Copy-paste debugging              ├── Complex call chains
├── No surprises                      ├── Debugging across files
└── Fast onboarding                   └── Cognitive overhead
```

## Decision Framework

### Rule of Three (or Ten)

```
Features Count          Action
─────────────────────────────────────────────────
1-10 features           Keep duplication ✅
11-20 features          Consider extraction
20+ features            Extract to shared module
```

### Change Frequency

```
Changes/Year            Action
─────────────────────────────────────────────────
0-2 changes             Safe to extract
3+ changes              Keep inline (unstable)
Pattern still evolving  NEVER extract
```

### Current Decision Matrix

```
┌─────────────┬──────────────┬────────────────┐
│  Dimension  │  Our Status  │  Recommendation│
├─────────────┼──────────────┼────────────────┤
│ # Features  │      8       │  Keep inline   │
│ Stability   │   Evolving   │  Keep inline   │
│ Pain Level  │     Low      │  Keep inline   │
└─────────────┴──────────────┴────────────────┘

✅ VERDICT: Keep duplication for now
```

## Future Migration Path

### Stage 1: Current (1-10 features)
**Status:** ✅ Active

```rust
// features/send_invite.rs
pub async fn handle(...) -> Result<...> {
    // 1. Extract cookie
    let token = jar.get("session")...
    
    // 2. Validate session (duplicated)
    let session = find_session(&pool, token).await?...
    
    // 3. Check permissions (duplicated)
    if !user.role.has_permission(...) { ... }
    
    // 4. Business logic
    send_invite(...)
}
```

**Pros:**
- ✅ Explicit and clear
- ✅ No hidden magic
- ✅ Easy to debug

**Cons:**
- ❌ ~6 lines duplicated per feature

### Stage 2: Axum Extractors (10-20 features)
**Status:** 🔮 Future (when needed)

```rust
// shared/auth.rs (NEW - single file)
pub struct Authenticated {
    pub user: User,
    pub pool: PgPool,
}

#[async_trait]
impl FromRequestParts<AppState> for Authenticated {
    async fn from_request_parts(...) -> Result<Self, AuthError> {
        // All auth logic in ONE place
        let token = parts.extract::<CookieJar>()...
        let session = find_session(&pool, token).await?;
        let user = find_user(&pool, session.user_id).await?;
        Ok(Authenticated { user, pool })
    }
}

// features/send_invite.rs (CLEANER)
pub async fn handle(
    auth: Authenticated,  // ← Auth extracted cleanly
    Json(req): Json<Request>,
) -> Result<...> {
    // Permission check still explicit
    if !auth.user.role.has_permission(...) { ... }
    
    // Business logic
    send_invite(&auth.pool, auth.user.id, ...)
}
```

**Migration Trigger:**
- 10+ features with identical auth pattern
- Auth pattern stabilized (no changes in 6 months)
- Team consensus that duplication is painful

**Pros:**
- ✅ Auth in one place
- ✅ Features stay clean
- ✅ Still explicit in handler signature

**Cons:**
- ❌ Adds one shared module
- ❌ Slightly less explicit

### Stage 3: Middleware (20+ features)
**Status:** 🔮 Far Future (maybe never)

```rust
// lib.rs
let protected_router = Router::new()
    .nest("/invites", features::send_invite::router())
    .nest("/users", features::list_users::router())
    .layer(AuthMiddleware);  // ← Transparent auth

// features/send_invite.rs (CLEANEST)
pub async fn handle(
    Extension(user): Extension<User>,  // ← Injected by middleware
    State(pool): State<PgPool>,
    Json(req): Json<Request>,
) -> Result<...> {
    // Just business logic
    send_invite(&pool, user.id, ...)
}
```

**Migration Trigger:**
- 20+ features
- Auth is 100% consistent across all
- Never changes

**Pros:**
- ✅ Completely transparent
- ✅ Zero duplication
- ✅ Single auth implementation

**Cons:**
- ❌ Hidden behavior
- ❌ Harder to debug
- ❌ Less explicit

## Recommended Timeline

```
Now                 6 months          12 months         24 months
│                   │                 │                 │
├─── Stage 1 ───────┼─────────────────┼─────────────────┤
     (Current)      │                 │                 │
     8 features     │                 │                 │
                    │                 │                 │
                    ├── Stage 2? ─────┼─────────────────┤
                    Evaluate:         │                 │
                    • 15+ features?   │                 │
                    • Stable pattern? │                 │
                    • Team consensus? │                 │
                                      │                 │
                                      ├─── Stage 3? ────┤
                                      Evaluate:         │
                                      • 25+ features?   │
                                      • Never changes?  │
```

## Key Principles

### 1. The Rule of Three (Modified for VSA)

> **Original:** Duplicate twice, extract on third occurrence  
> **VSA Version:** Duplicate ten times, extract on eleventh

Why? Because in VSA, duplication cost is LOW but abstraction cost is HIGH.

### 2. Sandi Metz Quote

> "Duplication is far cheaper than the wrong abstraction."

**Applied to VSA:**
- Duplication cost: 6 lines × $1/line = $6 per feature
- Wrong abstraction cost: Refactor 8 features × $100/feature = $800
- **Verdict:** Duplication is 133x cheaper

### 3. YAGNI (You Aren't Gonna Need It)

Don't abstract until you have **concrete evidence** of pain:
- ✅ Auth bug affected 10+ features
- ✅ Auth change required 20+ file edits
- ✅ New developer took 2+ days to understand auth

**Current evidence:** None of the above. Keep duplicating.

## Summary

### Current Status
- ✅ **8 features** with ~157 lines of duplication
- ✅ **Acceptable** duplication range
- ✅ **Action:** Keep as-is

### When to Revisit
- 📊 Reach 15+ features
- 🐛 Auth bug affects 5+ features
- 📅 6 months with stable auth pattern
- 👥 Team requests consolidation

### Next Steps
1. **Now:** Keep building features with duplication
2. **At 15 features:** Evaluate Stage 2 (extractors)
3. **At 25 features:** Evaluate Stage 3 (middleware)
4. **Always:** Prefer explicit over implicit

---

**Remember:** In VSA, duplication is a *feature*, not a bug. It enables independence, which is the core principle.
