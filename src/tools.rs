//! MCP tool domain handlers.

use crate::api::{self, ApiLookupError, ApiSymbol};
use crate::diagnostics::{DiagnosticsOutput, LeptosDiagnostics, render_diagnostics};
use crate::docs::{
    self, CatalogSection, CrateVersion, SectionLookupError, SectionSearchMatch,
    SnippetClassification,
};
use crate::recipes::{self, Recipe, RecipeLookupError};
use serde::Serialize;

pub const LIST_SECTIONS_TOOL: &str = "list-sections";
pub const GET_DOCUMENTATION_TOOL: &str = "get-documentation";
pub const LEPTOS_DIAGNOSTICS_TOOL: &str = "leptos-diagnostics";
pub const SEARCH_DOCS_TOOL: &str = "search-docs";
pub const LOOKUP_API_TOOL: &str = "lookup-api";
pub const LEPTOS_AXUM_RECIPE_TOOL: &str = "leptos-axum-recipe";
pub const MAX_DIAGNOSTIC_CODE_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    InvalidParams(String),
    UnknownTool(String),
    DocumentationLookup(SectionLookupError),
    ApiLookup(ApiLookupError),
    RecipeLookup(RecipeLookupError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SectionSummary {
    pub id: &'static str,
    pub title: &'static str,
    pub path: &'static str,
    pub use_cases: &'static str,
    pub aliases: &'static [&'static str],
    pub leptos_version: &'static str,
    pub source: &'static str,
    pub source_path: &'static str,
    pub reviewed_at: &'static str,
    pub resource_uri: String,
    pub crate_versions: &'static [CrateVersion],
    pub source_url: &'static str,
    pub task_tags: &'static [&'static str],
    pub crate_apis: &'static [&'static str],
    pub prerequisites: &'static [&'static str],
    pub common_errors: &'static [&'static str],
    pub related_sections: &'static [&'static str],
    pub snippet_classification: SnippetClassification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListSectionsOutput {
    pub sections: Vec<SectionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocumentationOutput {
    pub section: SectionSummary,
    pub content: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SectionSearchSummary {
    pub section: SectionSummary,
    pub score: usize,
    pub matched_fields: Vec<&'static str>,
    pub why: String,
    pub next_actions: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchDocsOutput {
    pub query: String,
    pub results: Vec<SectionSearchSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiLookupOutput {
    pub query: String,
    pub symbol: ApiSymbol,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecipeOutput {
    pub recipe: Recipe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum StructuredToolOutput {
    ListSections(ListSectionsOutput),
    Documentation(DocumentationOutput),
    Diagnostics(DiagnosticsOutput),
    SearchDocs(SearchDocsOutput),
    ApiLookup(ApiLookupOutput),
    Recipe(RecipeOutput),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutput {
    pub text: String,
    pub structured: StructuredToolOutput,
}

pub struct LeptosTools {}

impl LeptosTools {
    pub fn new() -> Self {
        Self {}
    }

    pub fn list_sections(&self) -> ToolOutput {
        let output = ListSectionsOutput {
            sections: docs::list_catalog_sections()
                .iter()
                .map(section_summary)
                .collect(),
        };
        let text = output
            .sections
            .iter()
            .map(|section| {
                format!(
                    "* id: {}, title: {}, use_cases: {}, aliases: {}",
                    section.id,
                    section.title,
                    section.use_cases,
                    section.aliases.join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        ToolOutput {
            text,
            structured: StructuredToolOutput::ListSections(output),
        }
    }

    pub fn get_documentation(&self, section: &str) -> Result<ToolOutput, ToolError> {
        let catalog_section =
            docs::get_catalog_section(section).map_err(ToolError::DocumentationLookup)?;
        let doc = catalog_section.section;
        let output = DocumentationOutput {
            section: section_summary(catalog_section),
            content: doc.content,
        };

        Ok(ToolOutput {
            text: format!("# {}\n\n{}", doc.title, doc.content),
            structured: StructuredToolOutput::Documentation(output),
        })
    }

    pub fn search_docs(&self, query: &str) -> Result<ToolOutput, ToolError> {
        let matches = docs::search_sections(query).map_err(ToolError::DocumentationLookup)?;
        let results: Vec<SectionSearchSummary> =
            matches.iter().map(section_search_summary).collect();
        let text = if results.is_empty() {
            format!("No documentation sections matched '{}'.", query.trim())
        } else {
            results
                .iter()
                .map(|result| {
                    format!(
                        "* id: {}, score: {}, why: {}",
                        result.section.id, result.score, result.why
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        Ok(ToolOutput {
            text,
            structured: StructuredToolOutput::SearchDocs(SearchDocsOutput {
                query: query.trim().to_string(),
                results,
            }),
        })
    }

    pub fn lookup_api(
        &self,
        query: &str,
        crate_name: Option<&str>,
    ) -> Result<ToolOutput, ToolError> {
        let symbol = api::lookup_symbol(query, crate_name).map_err(ToolError::ApiLookup)?;
        let output = ApiLookupOutput {
            query: query.trim().to_string(),
            symbol: *symbol,
        };

        Ok(ToolOutput {
            text: format!(
                "{} ({})\n{}\n{}",
                symbol.name, symbol.crate_name, symbol.summary, symbol.url
            ),
            structured: StructuredToolOutput::ApiLookup(output),
        })
    }

    pub fn leptos_axum_recipe(&self, recipe: &str) -> Result<ToolOutput, ToolError> {
        let recipe = recipes::get_recipe(recipe).map_err(ToolError::RecipeLookup)?;
        let output = RecipeOutput { recipe: *recipe };

        Ok(ToolOutput {
            text: render_recipe(recipe),
            structured: StructuredToolOutput::Recipe(output),
        })
    }

    pub fn diagnose_leptos_code(&self, code: &str) -> Result<ToolOutput, ToolError> {
        if code.trim().is_empty() {
            return Err(ToolError::InvalidParams(
                "code must be a non-empty string".to_string(),
            ));
        }
        if code.len() > MAX_DIAGNOSTIC_CODE_BYTES {
            return Err(ToolError::InvalidParams(format!(
                "code must be at most {MAX_DIAGNOSTIC_CODE_BYTES} bytes"
            )));
        }

        let output = LeptosDiagnostics::analyze(code);

        Ok(ToolOutput {
            text: render_diagnostics(&output),
            structured: StructuredToolOutput::Diagnostics(output),
        })
    }
}

impl Default for LeptosTools {
    fn default() -> Self {
        Self::new()
    }
}

fn section_summary(catalog_section: &CatalogSection) -> SectionSummary {
    let section = catalog_section.section;
    let metadata = catalog_section.metadata;

    SectionSummary {
        id: section.id,
        title: section.title,
        path: section.path,
        use_cases: section.use_cases,
        aliases: section.aliases,
        leptos_version: section.leptos_version,
        source: section.source,
        source_path: section.source_path,
        reviewed_at: section.reviewed_at,
        resource_uri: docs::resource_uri(section),
        crate_versions: metadata.crate_versions,
        source_url: metadata.source_url,
        task_tags: metadata.task_tags,
        crate_apis: metadata.crate_apis,
        prerequisites: metadata.prerequisites,
        common_errors: metadata.common_errors,
        related_sections: metadata.related_sections,
        snippet_classification: metadata.snippet_classification,
    }
}

fn section_search_summary(search_match: &SectionSearchMatch) -> SectionSearchSummary {
    SectionSearchSummary {
        section: section_summary(&CatalogSection {
            section: search_match.section,
            metadata: search_match.metadata,
        }),
        score: search_match.score,
        matched_fields: search_match.matched_fields.clone(),
        why: search_match.why.clone(),
        next_actions: search_match.next_actions.clone(),
    }
}

fn render_recipe(recipe: &Recipe) -> String {
    let mut text = format!("# {}\n\n{}\n\n", recipe.title, recipe.summary);
    text.push_str("Steps:\n");
    for (index, step) in recipe.steps.iter().enumerate() {
        text.push_str(&format!("{}. {}\n", index + 1, step));
    }
    text.push_str("\nValidation:\n");
    for item in recipe.validation {
        text.push_str(&format!("- {item}\n"));
    }
    text
}

impl ToolError {
    pub fn message(&self) -> String {
        match self {
            ToolError::InvalidParams(message) => message.clone(),
            ToolError::UnknownTool(_) => "Unknown tool".to_string(),
            ToolError::DocumentationLookup(SectionLookupError::Empty) => {
                "section must be a non-empty canonical id or alias".to_string()
            }
            ToolError::DocumentationLookup(SectionLookupError::Unknown { .. }) => {
                "Unknown documentation section".to_string()
            }
            ToolError::DocumentationLookup(SectionLookupError::Ambiguous { matches, .. }) => {
                format!(
                    "Ambiguous documentation section. Matching sections: {}",
                    matches.join(", ")
                )
            }
            ToolError::ApiLookup(ApiLookupError::Empty) => {
                "query must be a non-empty API symbol or alias".to_string()
            }
            ToolError::ApiLookup(ApiLookupError::Unknown { .. }) => {
                "Unknown API symbol".to_string()
            }
            ToolError::ApiLookup(ApiLookupError::Ambiguous { matches, .. }) => {
                format!(
                    "Ambiguous API symbol. Matching symbols: {}",
                    matches.join(", ")
                )
            }
            ToolError::RecipeLookup(RecipeLookupError::Empty) => {
                "recipe must be a non-empty recipe id or alias".to_string()
            }
            ToolError::RecipeLookup(RecipeLookupError::Unknown { .. }) => {
                "Unknown Leptos Axum recipe".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_sections_returns_structured_catalog_metadata() {
        let tools = LeptosTools::new();
        let output = tools.list_sections();

        match output.structured {
            StructuredToolOutput::ListSections(list) => {
                let signals = list
                    .sections
                    .iter()
                    .find(|section| section.id == "signals")
                    .expect("signals should be listed");
                assert!(signals.task_tags.contains(&"reactivity"));
                assert_eq!(signals.resource_uri, "leptos://docs/signals");
            }
            _ => panic!("expected list sections output"),
        }
    }

    #[test]
    fn search_docs_returns_task_oriented_results() {
        let tools = LeptosTools::new();
        let output = tools
            .search_docs("Axum state")
            .expect("search should succeed");

        match output.structured {
            StructuredToolOutput::SearchDocs(search) => {
                assert!(
                    search
                        .results
                        .iter()
                        .any(|result| result.section.id == "axum")
                );
                assert!(
                    search
                        .results
                        .iter()
                        .any(|result| !result.next_actions.is_empty())
                );
            }
            _ => panic!("expected search docs output"),
        }
    }

    #[test]
    fn lookup_api_returns_symbol_metadata() {
        let tools = LeptosTools::new();
        let output = tools
            .lookup_api("file_and_error_handler", Some("leptos_axum"))
            .expect("API symbol should resolve");

        match output.structured {
            StructuredToolOutput::ApiLookup(api) => {
                assert_eq!(api.symbol.name, "leptos_axum::file_and_error_handler");
                assert_eq!(api.symbol.version, "0.8.9");
            }
            _ => panic!("expected API lookup output"),
        }
    }

    #[test]
    fn lookup_api_renders_ambiguous_symbol_errors() {
        let tools = LeptosTools::new();
        let error = tools
            .lookup_api("extractor", None)
            .expect_err("ambiguous API query must fail");

        assert_eq!(
            error.message(),
            "Ambiguous API symbol. Matching symbols: leptos_axum::extract, leptos_axum::extract_with_state, axum::Json"
        );
    }

    #[test]
    fn leptos_axum_recipe_returns_steps_and_files() {
        let tools = LeptosTools::new();
        let output = tools
            .leptos_axum_recipe("state")
            .expect("state recipe should resolve");

        match output.structured {
            StructuredToolOutput::Recipe(recipe) => {
                assert_eq!(recipe.recipe.id, "state-context");
                assert!(!recipe.recipe.steps.is_empty());
                assert!(!recipe.recipe.files.is_empty());
            }
            _ => panic!("expected recipe output"),
        }
    }

    #[test]
    fn get_documentation_rejects_missing_section_identity() {
        let tools = LeptosTools::new();
        let error = tools
            .get_documentation("")
            .expect_err("empty section must fail");

        assert_eq!(
            error.message(),
            "section must be a non-empty canonical id or alias"
        );
    }

    #[test]
    fn diagnostics_reject_empty_code() {
        let tools = LeptosTools::new();
        let error = tools
            .diagnose_leptos_code("  ")
            .expect_err("empty code must fail");

        assert_eq!(error.message(), "code must be a non-empty string");
    }

    #[test]
    fn diagnostics_reject_oversized_code() {
        let tools = LeptosTools::new();
        let code = "x".repeat(MAX_DIAGNOSTIC_CODE_BYTES + 1);
        let error = tools
            .diagnose_leptos_code(&code)
            .expect_err("oversized code must fail");

        assert_eq!(
            error.message(),
            format!("code must be at most {MAX_DIAGNOSTIC_CODE_BYTES} bytes")
        );
    }
}
