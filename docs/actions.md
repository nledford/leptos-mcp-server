# Actions for Mutations

Actions wrap async operations that **mutate data** (POST, PUT, DELETE). Unlike Resources (which load data), Actions are triggered explicitly.

## Creating an Action

```rust
#[server(CreateTodo)]
pub async fn create_todo(title: String) -> Result<Todo, ServerFnError> {
    let pool = expect_context::<SqlitePool>();
    // Insert into database...
    Ok(new_todo)
}

#[component]
fn TodoApp() -> impl IntoView {
    // Create action from server function
    let add_todo = ServerAction::<CreateTodo>::new();

    // Or create a generic action
    let action = Action::new(|input: &String| {
        let input = input.clone();
        async move { create_todo(input).await }
    });

    // ...
}
```

## Action State

Actions provide reactive state:

```rust
let action = ServerAction::<CreateTodo>::new();

// Is the action currently running?
let pending = action.pending();

// Get the result after completion
let value = action.value();

// Get the input that was submitted
let input = action.input();

// Version counter (increments each submission)
let version = action.version();
```

## Using with ActionForm

`<ActionForm>` provides progressive enhancement:

```rust
#[component]
fn CreateTodoForm() -> impl IntoView {
    let action = ServerAction::<CreateTodo>::new();
    let pending = action.pending();

    view! {
        <ActionForm action=action>
            <input
                type="text"
                name="title"  // Must match server fn param
                placeholder="New todo"
            />
            <button type="submit" disabled=pending>
                {move || if pending.get() { "Adding..." } else { "Add" }}
            </button>
        </ActionForm>

        // Show result
        {move || action.value().get().map(|result| match result {
            Ok(todo) => view! { <p>"Created: " {todo.title}</p> }.into_any(),
            Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
        })}
    }
}
```

## Manual Dispatch

```rust
let action = Action::new(|data: &String| {
    let data = data.clone();
    async move { save_data(data).await }
});

view! {
    <button on:click=move |_| {
        action.dispatch("Hello".to_string());
    }>
        "Save"
    </button>
}
```

## Action vs Resource

| Feature       | Resource                  | Action                      |
| ------------- | ------------------------- | --------------------------- |
| Trigger       | Automatic (signal change) | Manual (dispatch/form)      |
| Use Case      | Loading data              | Mutating data               |
| Pending state | Inside `<Suspense>`       | `.pending()` signal         |
| Result access | `.get()` returns Option   | `.value()` after completion |

## Combining with Resources

Refetch resources after successful action:

```rust
let todos = Resource::new(|| (), |_| fetch_todos());
let add_todo = ServerAction::<CreateTodo>::new();

// Refetch todos when action completes
Effect::new(move |_| {
    if add_todo.value().get().is_some() {
        todos.refetch();
    }
});

view! {
    <ActionForm action=add_todo>
        <input type="text" name="title" />
        <button type="submit">"Add"</button>
    </ActionForm>

    <Suspense fallback=|| "Loading...">
        <ul>
            {move || todos.get().map(|list| {
                list.into_iter().map(|t| view! { <li>{t.title}</li> }).collect_view()
            })}
        </ul>
    </Suspense>
}
```

## Database Mutations

Database-backed actions should delegate writes to server functions. Use the
shared pool from server context and wrap multi-step changes in a transaction.

```rust
#[server(ArchiveTodo)]
pub async fn archive_todo(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<SqlitePool>();
    let mut tx = pool
        .begin()
        .await
        .map_err(|error| {
            tracing::error!(?error, "could not start transaction");
            ServerFnError::new("Could not start transaction")
        })?;

    sqlx::query!("UPDATE todos SET archived = TRUE WHERE id = ?", id)
        .execute(&mut *tx)
        .await
        .map_err(|error| {
            tracing::error!(?error, "could not archive todo");
            ServerFnError::new("Could not archive todo")
        })?;

    tx.commit()
        .await
        .map_err(|error| {
            tracing::error!(?error, "could not commit transaction");
            ServerFnError::new("Could not commit transaction")
        })?;

    Ok(())
}
```

Prefer `query!` / `query_as!` for fixed writes and SeaQuery only when the update
shape is genuinely dynamic. Keep all user input bound as parameters or SeaQuery
values, then refetch affected `Resource`s after successful actions.

## Local Actions

For `!Send` types or no SSR:

```rust
// For !Send types (browser APIs)
let action = Action::new_local(|_: &()| {
    async move {
        // Browser-only operation
    }
});

// For !Sync closures
let action = Action::new_unsync(|_: &()| {
    async move { /* ... */ }
});
```

## Error Handling

```rust
let action = ServerAction::<CreateTodo>::new();

view! {
    <ActionForm action=action>
        // ... form fields
    </ActionForm>

    // Error display
    {move || action.value().get().and_then(|r| r.err()).map(|e| {
        view! { <p class="error">{e.to_string()}</p> }
    })}
}
```

## Complete Example

```rust
#[server(SaveSettings)]
pub async fn save_settings(theme: String, lang: String) -> Result<(), ServerFnError> {
    // Save to database
    Ok(())
}

#[component]
fn SettingsForm() -> impl IntoView {
    let action = ServerAction::<SaveSettings>::new();
    let pending = action.pending();
    let saved = move || action.value().get().map(|r| r.is_ok()).unwrap_or(false);

    view! {
        <ActionForm action=action>
            <select name="theme">
                <option value="light">"Light"</option>
                <option value="dark">"Dark"</option>
            </select>
            <select name="lang">
                <option value="en">"English"</option>
                <option value="vi">"Vietnamese"</option>
            </select>
            <button type="submit" disabled=pending>
                {move || if pending.get() { "Saving..." } else { "Save" }}
            </button>
        </ActionForm>

        <Show when=saved>
            <p class="success">"Settings saved!"</p>
        </Show>
    }
}
```
