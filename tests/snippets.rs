use leptos_mcp_server::api::rust_api_snippets;
use leptos_mcp_server::docs::{SnippetClassification, get_section, rust_code_blocks};
use leptos_mcp_server::recipes::rust_recipe_snippets;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SHARED_PRELUDE: &str = r#"
mod leptos_axum {
    use super::Handler;

    pub fn file_and_error_handler<T>(_handler: T) -> Handler {
        Handler
    }
}

#[derive(Clone, Copy)]
struct App;

#[derive(Clone, Copy)]
struct Handler;

#[derive(Clone, Copy)]
struct LeptosOptions;

#[derive(Clone, Copy)]
struct Routes;

#[derive(Clone, Copy)]
struct Router;

impl Router {
    fn new() -> Self {
        Self
    }

    fn route(self, _path: &str, _handler: Handler) -> Self {
        self
    }

    fn leptos_routes(self, _options: &LeptosOptions, _routes: Routes, _app: App) -> Self {
        self
    }

    fn leptos_routes_with_context<T>(
        self,
        _options: &LeptosOptions,
        _routes: Routes,
        _provide_context: T,
        _app: App,
    ) -> Self {
        self
    }

    fn fallback(self, _handler: Handler) -> Self {
        self
    }
}

fn post<T>(_handler: T) -> Handler {
    Handler
}

fn file_and_error_handler<T>(handler: T) -> Handler {
    leptos_axum::file_and_error_handler(handler)
}

fn generate_route_list(_app: App) -> Routes {
    Routes
}

fn handle_server_fns() {}

fn provide_context() {}

fn shell() {}
"#;

#[test]
fn compile_candidate_docs_snippets_compile_with_shared_harness() {
    let candidates: Vec<_> = rust_code_blocks()
        .into_iter()
        .filter(|block| block.classification == SnippetClassification::CompileCandidate)
        .collect();

    assert!(
        !candidates.is_empty(),
        "expected at least one docs compile candidate"
    );

    for (index, block) in candidates.iter().enumerate() {
        let section = get_section(block.section_id).expect("docs snippet section should exist");
        compile_snippet(
            &format!("doc-{}-{index}", block.section_id),
            &block.content,
            SnippetDiagnostic {
                kind: "docs section",
                id_label: "section id",
                id: block.section_id,
                source_path: section.source_path,
                classification: block.classification,
            },
        );
    }
}

#[test]
fn compile_candidate_recipe_snippets_compile_with_shared_harness() {
    let candidates: Vec<_> = rust_recipe_snippets()
        .into_iter()
        .filter(|snippet| snippet.classification == SnippetClassification::CompileCandidate)
        .collect();

    assert!(
        !candidates.is_empty(),
        "expected at least one recipe compile candidate"
    );

    for (index, snippet) in candidates.iter().enumerate() {
        compile_snippet(
            &format!("recipe-{}-{index}", snippet.recipe_id),
            snippet.content,
            SnippetDiagnostic {
                kind: "recipe",
                id_label: "recipe id",
                id: snippet.recipe_id,
                source_path: snippet.path,
                classification: snippet.classification,
            },
        );
    }
}

#[test]
fn compile_candidate_api_snippets_compile_with_shared_harness() {
    let candidates: Vec<_> = rust_api_snippets()
        .into_iter()
        .filter(|snippet| snippet.classification == SnippetClassification::CompileCandidate)
        .collect();

    assert!(
        !candidates.is_empty(),
        "expected at least one API compile candidate"
    );

    for (index, snippet) in candidates.iter().enumerate() {
        compile_snippet(
            &format!("api-{}-{index}", snippet.symbol_name),
            snippet.content,
            SnippetDiagnostic {
                kind: "API catalog snippet",
                id_label: "symbol name",
                id: snippet.symbol_name,
                source_path: "src/api.rs",
                classification: snippet.classification,
            },
        );
    }
}

#[derive(Clone, Copy)]
struct SnippetDiagnostic {
    kind: &'static str,
    id_label: &'static str,
    id: &'static str,
    source_path: &'static str,
    classification: SnippetClassification,
}

fn compile_snippet(name: &str, snippet: &str, diagnostic: SnippetDiagnostic) {
    let source_path = write_compile_unit(name, snippet);
    let output_path = source_path.with_extension("bin");
    let output = Command::new("rustc")
        .arg("--edition=2024")
        .arg("--crate-name")
        .arg(sanitize_crate_name(name))
        .arg("--out-dir")
        .arg(
            source_path
                .parent()
                .expect("source path should have parent"),
        )
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("rustc should run for snippet compile check");

    assert!(
        output.status.success(),
        "snippet '{name}' failed to compile\nkind: {}\n{}: {}\nsource path: {}\nclassification: {:?}\ngenerated fixture: {}\nstdout:\n{}\nstderr:\n{}",
        diagnostic.kind,
        diagnostic.id_label,
        diagnostic.id,
        diagnostic.source_path,
        diagnostic.classification,
        source_path.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_compile_unit(name: &str, snippet: &str) -> PathBuf {
    let dir = compile_dir().join(name);
    fs::create_dir_all(&dir).expect("snippet compile temp dir should be creatable");

    let source = format!(
        "{SHARED_PRELUDE}\nfn main() {{\n    let leptos_options = LeptosOptions;\n    let routes = Routes;\n    let app = App;\n{snippet}\n}}\n"
    );
    let path = dir.join("main.rs");
    fs::write(&path, source).expect("snippet compile unit should be writable");
    path
}

fn compile_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("snippet-checks")
        .join(std::process::id().to_string())
}

fn sanitize_crate_name(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}
