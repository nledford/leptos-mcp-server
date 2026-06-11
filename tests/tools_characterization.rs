use leptos_mcp_server::tools::{
    GET_DOCUMENTATION_TOOL, LEPTOS_AXUM_RECIPE_TOOL, LEPTOS_DIAGNOSTICS_TOOL, LIST_SECTIONS_TOOL,
    LOOKUP_API_TOOL, LeptosTools, MAX_DIAGNOSTIC_CODE_BYTES, SEARCH_DOCS_TOOL,
    StructuredToolOutput,
};

#[test]
fn tool_name_constants_match_current_mcp_tool_names() {
    assert_eq!(LIST_SECTIONS_TOOL, "list-sections");
    assert_eq!(GET_DOCUMENTATION_TOOL, "get-documentation");
    assert_eq!(LEPTOS_DIAGNOSTICS_TOOL, "leptos-diagnostics");
    assert_eq!(SEARCH_DOCS_TOOL, "search-docs");
    assert_eq!(LOOKUP_API_TOOL, "lookup-api");
    assert_eq!(LEPTOS_AXUM_RECIPE_TOOL, "leptos-axum-recipe");
}

#[test]
fn list_sections_characterizes_structured_catalog_and_text_summary() {
    let output = LeptosTools::new().list_sections();

    assert!(output.text.contains("* id: signals"));
    assert!(output.text.contains("aliases: signal, reactivity, state"));

    let StructuredToolOutput::ListSections(list) = output.structured else {
        panic!("expected list-sections structured variant");
    };

    let signals = list
        .sections
        .iter()
        .find(|section| section.id == "signals")
        .expect("signals should stay in the catalog");
    assert_eq!(signals.title, "Signals");
    assert_eq!(signals.resource_uri, "leptos://docs/signals");
    assert!(signals.task_tags.contains(&"reactivity"));
    assert!(
        signals
            .crate_versions
            .iter()
            .any(|version| { version.name == "leptos" && version.version == "0.8.19" })
    );
}

#[test]
fn get_documentation_characterizes_section_output_text_and_lookup_errors() {
    let tools = LeptosTools::new();
    let output = tools
        .get_documentation("signal")
        .expect("signal alias should resolve");

    assert!(output.text.starts_with("# Signals\n\n"));
    assert!(output.text.contains("signal"));

    let StructuredToolOutput::Documentation(documentation) = output.structured else {
        panic!("expected get-documentation structured variant");
    };
    assert_eq!(documentation.section.id, "signals");
    assert_eq!(documentation.section.resource_uri, "leptos://docs/signals");
    assert!(documentation.content.contains("signal"));

    assert_eq!(
        tools
            .get_documentation(" ")
            .expect_err("blank section should be rejected")
            .message(),
        "section must be a non-empty canonical id or alias"
    );
    assert_eq!(
        tools
            .get_documentation("missing-section")
            .expect_err("unknown section should be rejected")
            .message(),
        "Unknown documentation section: missing-section"
    );
}

#[test]
fn leptos_diagnostics_characterizes_structured_diagnostics_text_and_size_guard() {
    let tools = LeptosTools::new();
    let code = r#"
fn App() -> impl IntoView {
    let count = signal(0);
    view! { <p>{count.get()}</p> }
}
"#;
    let output = tools
        .diagnose_leptos_code(code)
        .expect("diagnostics should analyze non-empty code");

    assert!(output.text.contains("WARNING [leptos.signal-get-in-view]"));
    assert!(
        output
            .text
            .contains("WARNING [leptos.missing-component-attribute]")
    );

    let StructuredToolOutput::Diagnostics(diagnostics) = output.structured else {
        panic!("expected leptos-diagnostics structured variant");
    };
    assert_eq!(diagnostics.summary.warning_count, 2);
    assert_eq!(diagnostics.summary.error_count, 0);
    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.rule_id == "leptos.signal-get-in-view"
            && diagnostic.message
                == "Reactive signal reads inside views should be wrapped in `move ||`."
            && diagnostic.suggested_fix
                == Some("Use `{move || value.get()}` for reactive view updates.")
    }));

    assert_eq!(
        tools
            .diagnose_leptos_code("\n\t")
            .expect_err("blank code should be rejected")
            .message(),
        "code must be a non-empty string"
    );
    assert_eq!(
        tools
            .diagnose_leptos_code(&"x".repeat(MAX_DIAGNOSTIC_CODE_BYTES + 1))
            .expect_err("oversized diagnostic payload should be rejected")
            .message(),
        format!("code must be at most {MAX_DIAGNOSTIC_CODE_BYTES} bytes")
    );
}

#[test]
fn search_docs_characterizes_ranked_results_text_and_empty_query_error() {
    let tools = LeptosTools::new();
    let output = tools
        .search_docs("Axum state")
        .expect("search should return ranked docs");

    assert!(output.text.contains("* id: axum"));
    assert!(output.text.contains("score:"));
    assert!(output.text.contains("why:"));

    let StructuredToolOutput::SearchDocs(search) = output.structured else {
        panic!("expected search-docs structured variant");
    };
    assert_eq!(search.query, "Axum state");
    let axum = search
        .results
        .iter()
        .find(|result| result.section.id == "axum")
        .expect("axum should match Axum state");
    assert!(axum.score > 0);
    assert!(!axum.matched_fields.is_empty());
    assert!(!axum.next_actions.is_empty());

    assert_eq!(
        tools
            .search_docs(" ")
            .expect_err("blank search should be rejected")
            .message(),
        "section must be a non-empty canonical id or alias"
    );
}

#[test]
fn lookup_api_characterizes_symbol_text_structured_output_and_errors() {
    let tools = LeptosTools::new();
    let output = tools
        .lookup_api("file_and_error_handler", Some("leptos_axum"))
        .expect("known leptos_axum API should resolve");

    assert!(
        output
            .text
            .contains("leptos_axum::file_and_error_handler (leptos_axum)")
    );
    assert!(output.text.contains("Convenience Axum handler"));
    assert!(
        output.text.contains(
            "https://docs.rs/leptos_axum/latest/leptos_axum/fn.file_and_error_handler.html"
        )
    );

    let StructuredToolOutput::ApiLookup(api) = output.structured else {
        panic!("expected lookup-api structured variant");
    };
    assert_eq!(api.query, "file_and_error_handler");
    assert_eq!(api.symbol.name, "leptos_axum::file_and_error_handler");
    assert_eq!(api.symbol.kind, "function");
    assert_eq!(api.symbol.version, "0.8.9");
    assert!(
        api.symbol
            .related_sections
            .contains(&"ssr-hydration-deployment")
    );

    assert_eq!(
        tools
            .lookup_api(" ", None)
            .expect_err("blank API lookup should be rejected")
            .message(),
        "query must be a non-empty API symbol or alias"
    );
    assert_eq!(
        tools
            .lookup_api("nope", Some("leptos"))
            .expect_err("unknown API lookup should be rejected")
            .message(),
        "Unknown API symbol in crate leptos: nope"
    );
    assert_eq!(
        tools
            .lookup_api("extractor", None)
            .expect_err("ambiguous API lookup should be rejected")
            .message(),
        "Ambiguous API symbol 'extractor'. Matching symbols: leptos_axum::extract, leptos_axum::extract_with_state, axum::Json"
    );
}

#[test]
fn leptos_axum_recipe_characterizes_recipe_text_structured_output_and_errors() {
    let tools = LeptosTools::new();
    let output = tools
        .leptos_axum_recipe("state")
        .expect("state recipe alias should resolve");

    assert!(
        output
            .text
            .starts_with("# Share Axum state with server functions\n\n")
    );
    assert!(output.text.contains("Steps:\n"));
    assert!(output.text.contains("Validation:\n"));

    let StructuredToolOutput::Recipe(recipe) = output.structured else {
        panic!("expected leptos-axum-recipe structured variant");
    };
    assert_eq!(recipe.recipe.id, "state-context");
    assert!(recipe.recipe.crates.contains(&"leptos_axum 0.8.9"));
    assert!(
        recipe
            .recipe
            .related_apis
            .contains(&"leptos_axum::extract_with_state")
    );
    assert!(recipe.recipe.files.iter().any(|file| {
        file.path == "src/main.rs" && file.content.contains("leptos_routes_with_context")
    }));

    assert_eq!(
        tools
            .leptos_axum_recipe(" ")
            .expect_err("blank recipe should be rejected")
            .message(),
        "recipe must be a non-empty recipe id or alias"
    );
    assert_eq!(
        tools
            .leptos_axum_recipe("missing-recipe")
            .expect_err("unknown recipe should be rejected")
            .message(),
        "Unknown Leptos Axum recipe: missing-recipe"
    );
}
