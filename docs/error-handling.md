# Error Handling in Leptos

## ErrorBoundary Component

Catch and display errors gracefully:

```rust
use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <div class="error-container">
                    <h2>"Something went wrong"</h2>
                    <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                            .collect_view()
                        }
                    </ul>
                </div>
            }
        }>
            <MyComponent />
        </ErrorBoundary>
    }
}
```

## Server Function Errors

```rust
use leptos::server_fn::ServerFnError;

#[server(GetUser)]
pub async fn get_user(id: String) -> Result<User, ServerFnError> {
    // Return error with message
    if id.is_empty() {
        return Err(ServerFnError::new("ID cannot be empty"));
    }

    // Database error handling
    let user = db.find(&id).await
        .map_err(|error| {
            tracing::error!(?error, "database lookup failed");
            ServerFnError::new("Database lookup failed")
        })?;

    user.ok_or_else(|| ServerFnError::new("User not found"))
}
```

## Custom Error Types

```rust
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error")]
    Internal,
}

// Convert to ServerFnError
impl From<AppError> for ServerFnError {
    fn from(err: AppError) -> Self {
        ServerFnError::new(err.to_string())
    }
}
```

## Database Error Boundaries

Map `sqlx::Error` or SeaQuery/sqlx adapter errors into application errors at the
server boundary. Log detailed database failures server-side, but return stable,
user-safe messages to Leptos components.

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Item not found")]
    NotFound,

    #[error("Database operation failed")]
    Database(#[from] sqlx::Error),
}

impl From<AppError> for ServerFnError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::NotFound => ServerFnError::new("Item not found"),
            AppError::Database(_) => ServerFnError::new("Database operation failed"),
        }
    }
}
```

Handle expected cases such as `RowNotFound`, unique constraint failures, and
validation errors explicitly. Treat unexpected database errors as internal
failures and avoid exposing SQL text, connection strings, or driver details to
the browser.

## Handling Resource Errors

```rust
#[component]
fn UserProfile(id: String) -> impl IntoView {
    let user = Resource::new(move || id.clone(), get_user);

    view! {
        <Suspense fallback=|| "Loading...">
            {move || user.get().map(|result| {
                match result {
                    Ok(user) => view! {
                        <div class="profile">
                            <h1>{user.name}</h1>
                        </div>
                    }.into_any(),
                    Err(e) => view! {
                        <div class="error">
                            <p>"Error: " {e.to_string()}</p>
                            <button on:click=move |_| user.refetch()>
                                "Retry"
                            </button>
                        </div>
                    }.into_any(),
                }
            })}
        </Suspense>
    }
}
```

## Best Practices

1. Always wrap risky components with `<ErrorBoundary>`
2. Use `thiserror` for custom error types
3. Provide user-friendly error messages
4. Include retry functionality for recoverable errors
5. Log detailed errors server-side, show simple messages client-side
6. Do not expose SQL statements, connection strings, or raw driver errors in
   browser-visible `ServerFnError` messages
