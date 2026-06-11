use leptos_mcp_server::{docs, prompts};

#[test]
fn docs_resources_enumerate_concrete_section_uris() {
    let resources = docs::list_sections()
        .iter()
        .map(|section| {
            (
                section.id,
                docs::resource_uri(section),
                section.title,
                section.use_cases,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(resources.len(), docs::list_catalog_sections().len());
    assert!(resources.len() >= 10, "expected the embedded docs catalog");

    for (id, uri, title, use_cases) in &resources {
        assert_eq!(uri, &format!("leptos://docs/{id}"));
        assert!(!title.trim().is_empty(), "resource {id} has a title");
        assert!(
            !use_cases.trim().is_empty(),
            "resource {id} has a description/use case summary"
        );
    }

    assert!(
        resources
            .iter()
            .any(|(id, uri, _, _)| *id == "axum" && uri == "leptos://docs/axum")
    );
    assert!(
        resources
            .iter()
            .any(|(id, uri, _, _)| *id == "resources" && uri == "leptos://docs/resources")
    );
}

#[test]
fn docs_resources_read_concrete_uris_and_report_missing_sections() {
    let axum = docs::get_catalog_section_by_resource_uri("leptos://docs/axum")
        .expect("concrete axum resource URI should resolve");
    assert_eq!(axum.section.id, "axum");
    assert_eq!(axum.section.title, "Axum 0.8.9 for Leptos Servers");
    assert!(axum.section.content.starts_with("# Axum"));
    assert_eq!(axum.metadata.id, "axum");

    let resources = docs::get_section_by_resource_uri("leptos://docs/resources")
        .expect("concrete resources URI should resolve");
    assert_eq!(resources.id, "resources");
    assert!(resources.content.contains("Resource"));

    assert_eq!(
        docs::get_section_by_resource_uri("leptos://docs/missing-section")
            .expect_err("missing concrete resource should be rejected"),
        docs::SectionLookupError::Unknown {
            query: "missing-section".to_string()
        }
    );
    assert_eq!(
        docs::get_section_by_resource_uri("docs/axum")
            .expect_err("resource URI without leptos docs prefix should be rejected"),
        docs::SectionLookupError::Unknown {
            query: "docs/axum".to_string()
        }
    );
}

#[test]
fn prompts_list_and_get_all_current_static_prompts() {
    let expected_names = [
        "wire-leptos-axum-ssr",
        "add-server-function",
        "review-sql-access",
        "debug-hydration",
        "review-axum-integration",
    ];

    let prompt_names = prompts::all_prompts()
        .iter()
        .map(|prompt| prompt.name)
        .collect::<Vec<_>>();
    assert_eq!(prompt_names, expected_names);

    for name in expected_names {
        let prompt = prompts::get_prompt(name).expect("static prompt should resolve by name");
        assert_eq!(prompt.name, name);
        assert!(!prompt.description.trim().is_empty());
        assert!(!prompt.related_tools.is_empty());
        assert!(!prompt.related_sections.is_empty());
        assert!(
            prompt
                .arguments
                .iter()
                .all(|argument| prompt.template.contains(&format!("{{{}}}", argument.name))),
            "prompt {name} template should mention every declared argument"
        );
    }
}

#[test]
fn prompts_get_normalizes_names_and_reports_missing_prompts() {
    assert_eq!(
        prompts::get_prompt("review_sql_access")
            .expect("underscored prompt name should normalize")
            .name,
        "review-sql-access"
    );
    assert_eq!(
        prompts::get_prompt(" Debug Hydration ")
            .expect("spaced prompt name should normalize")
            .name,
        "debug-hydration"
    );

    assert_eq!(
        prompts::get_prompt(" ").expect_err("blank prompt name should be rejected"),
        prompts::PromptLookupError::Empty
    );
    assert_eq!(
        prompts::get_prompt("missing-prompt").expect_err("missing prompt should be rejected"),
        prompts::PromptLookupError::Unknown {
            name: "missing-prompt".to_string()
        }
    );
}
