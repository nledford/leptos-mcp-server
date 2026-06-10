//! MCP tool domain handlers.

use crate::diagnostics::{render_diagnostics, DiagnosticsOutput, LeptosDiagnostics};
use crate::docs::{self, DocSection, SectionLookupError};
use serde::Serialize;

pub const LIST_SECTIONS_TOOL: &str = "list-sections";
pub const GET_DOCUMENTATION_TOOL: &str = "get-documentation";
pub const LEPTOS_DIAGNOSTICS_TOOL: &str = "leptos-diagnostics";
pub const MAX_DIAGNOSTIC_CODE_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    InvalidParams(String),
    UnknownTool(String),
    DocumentationLookup(SectionLookupError),
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
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum StructuredToolOutput {
    ListSections(ListSectionsOutput),
    Documentation(DocumentationOutput),
    Diagnostics(DiagnosticsOutput),
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
            sections: docs::list_sections().iter().map(section_summary).collect(),
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
        let doc = docs::get_section(section).map_err(ToolError::DocumentationLookup)?;
        let output = DocumentationOutput {
            section: section_summary(doc),
            content: doc.content,
        };

        Ok(ToolOutput {
            text: format!("# {}\n\n{}", doc.title, doc.content),
            structured: StructuredToolOutput::Documentation(output),
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

fn section_summary(section: &DocSection) -> SectionSummary {
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
    }
}

impl ToolError {
    pub fn message(&self) -> String {
        match self {
            ToolError::InvalidParams(message) => message.clone(),
            ToolError::UnknownTool(name) => format!("Unknown tool: {name}"),
            ToolError::DocumentationLookup(SectionLookupError::Empty) => {
                "section must be a non-empty canonical id or alias".to_string()
            }
            ToolError::DocumentationLookup(SectionLookupError::Unknown { query }) => {
                format!("Unknown documentation section: {query}")
            }
            ToolError::DocumentationLookup(SectionLookupError::Ambiguous { query, matches }) => {
                format!(
                    "Ambiguous documentation section '{query}'. Matching sections: {}",
                    matches.join(", ")
                )
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
                assert!(list.sections.iter().any(|section| section.id == "signals"));
            }
            _ => panic!("expected list sections output"),
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
