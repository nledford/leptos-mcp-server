//! Leptos code diagnostics.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub message: &'static str,
    pub span: SourceSpan,
    pub confidence: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_fix: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticSummary {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticsOutput {
    pub diagnostics: Vec<Diagnostic>,
    pub summary: DiagnosticSummary,
}

pub struct LeptosDiagnostics;

impl LeptosDiagnostics {
    pub fn analyze(code: &str) -> DiagnosticsOutput {
        let searchable = sanitize_rust_like_code(code);
        let mut diagnostics = Vec::new();

        diagnostics.extend(detect_signal_get_in_view(&searchable));
        diagnostics.extend(detect_signal_destructuring(&searchable));
        diagnostics.extend(detect_missing_component_attribute(&searchable));
        diagnostics.extend(detect_server_fn_error(&searchable));
        diagnostics.extend(detect_deprecated_create_signal(&searchable));

        let summary = DiagnosticSummary {
            error_count: diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count(),
            warning_count: diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count(),
            info_count: diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Info)
                .count(),
        };

        DiagnosticsOutput {
            diagnostics,
            summary,
        }
    }
}

fn detect_signal_get_in_view(searchable: &str) -> Vec<Diagnostic> {
    if !searchable.contains("view!") {
        return Vec::new();
    }

    searchable
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let column = line.find(".get()")?;
            if line.contains("move ||") {
                return None;
            }

            Some(Diagnostic {
                rule_id: "leptos.signal-get-in-view",
                severity: Severity::Error,
                message: "Reactive signal reads inside views should be wrapped in `move ||`.",
                span: SourceSpan {
                    line: index + 1,
                    column: column + 1,
                },
                confidence: Confidence::Medium,
                suggested_fix: Some("Use `{move || value.get()}` for reactive view updates."),
            })
        })
        .collect()
}

fn detect_signal_destructuring(searchable: &str) -> Vec<Diagnostic> {
    searchable
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let column = line.find("let signal =")?;
            Some(Diagnostic {
                rule_id: "leptos.signal-destructuring",
                severity: Severity::Warning,
                message: "Signals are clearer when destructured into getter and setter bindings.",
                span: SourceSpan {
                    line: index + 1,
                    column: column + 1,
                },
                confidence: Confidence::Medium,
                suggested_fix: Some("Use `let (getter, setter) = signal(value);`."),
            })
        })
        .collect()
}

fn detect_missing_component_attribute(searchable: &str) -> Vec<Diagnostic> {
    let lines: Vec<&str> = searchable.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let column = line.find("-> impl IntoView")?;
            if has_component_attribute_before(&lines, index) {
                return None;
            }

            Some(Diagnostic {
                rule_id: "leptos.missing-component-attribute",
                severity: Severity::Error,
                message:
                    "Functions returning `impl IntoView` should be annotated with `#[component]`.",
                span: SourceSpan {
                    line: index + 1,
                    column: column + 1,
                },
                confidence: Confidence::High,
                suggested_fix: Some("Add `#[component]` immediately above the component function."),
            })
        })
        .collect()
}

fn has_component_attribute_before(lines: &[&str], function_line_index: usize) -> bool {
    lines[..function_line_index]
        .iter()
        .rev()
        .map(|line| line.trim())
        .take_while(|line| line.is_empty() || line.starts_with("#["))
        .any(|line| line == "#[component]")
}

fn detect_server_fn_error(searchable: &str) -> Vec<Diagnostic> {
    let lines: Vec<&str> = searchable.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let column = line.find("#[server")?;
            let signature = lines
                .iter()
                .skip(index)
                .take(5)
                .copied()
                .collect::<Vec<_>>()
                .join(" ");

            if signature.contains("ServerFnError") {
                return None;
            }

            Some(Diagnostic {
                rule_id: "leptos.server-fn-error",
                severity: Severity::Info,
                message: "Server functions should return `Result<T, ServerFnError>`.",
                span: SourceSpan {
                    line: index + 1,
                    column: column + 1,
                },
                confidence: Confidence::Medium,
                suggested_fix: Some("Return `Result<T, ServerFnError>` from server functions."),
            })
        })
        .collect()
}

fn detect_deprecated_create_signal(searchable: &str) -> Vec<Diagnostic> {
    searchable
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let column = line.find("create_signal")?;
            Some(Diagnostic {
                rule_id: "leptos.deprecated-create-signal",
                severity: Severity::Info,
                message: "In Leptos 0.8+, prefer `signal()` over `create_signal()`.",
                span: SourceSpan {
                    line: index + 1,
                    column: column + 1,
                },
                confidence: Confidence::High,
                suggested_fix: Some("Use `signal(value)` instead of `create_signal(value)`."),
            })
        })
        .collect()
}

fn sanitize_rust_like_code(code: &str) -> String {
    let mut output = String::with_capacity(code.len());
    let mut chars = code.chars().peekable();
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                output.push('\n');
            } else {
                output.push(' ');
            }
            continue;
        }

        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                output.push(' ');
                output.push(' ');
                chars.next();
                in_block_comment = false;
            } else if ch == '\n' {
                output.push('\n');
            } else {
                output.push(' ');
            }
            continue;
        }

        if in_string {
            if ch == '\n' {
                in_string = false;
                output.push('\n');
            } else if ch == '"' && !escaped {
                in_string = false;
                output.push(' ');
            } else {
                escaped = ch == '\\' && !escaped;
                if ch != '\\' {
                    escaped = false;
                }
                output.push(' ');
            }
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            output.push(' ');
            output.push(' ');
            chars.next();
            in_line_comment = true;
        } else if ch == '/' && chars.peek() == Some(&'*') {
            output.push(' ');
            output.push(' ');
            chars.next();
            in_block_comment = true;
        } else if ch == '"' {
            output.push(' ');
            in_string = true;
            escaped = false;
        } else {
            output.push(ch);
        }
    }

    output
}

pub fn render_diagnostics(output: &DiagnosticsOutput) -> String {
    if output.diagnostics.is_empty() {
        return "No Leptos diagnostics found.".to_string();
    }

    output
        .diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{} [{}] line {}, column {}: {}",
                match diagnostic.severity {
                    Severity::Error => "ERROR",
                    Severity::Warning => "WARNING",
                    Severity::Info => "INFO",
                },
                diagnostic.rule_id,
                diagnostic.span.line,
                diagnostic.span.column,
                diagnostic.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_direct_signal_get_in_view_even_when_other_move_closure_exists() {
        let output = LeptosDiagnostics::analyze(
            r#"
            #[component]
            fn Counter() -> impl IntoView {
                let double = move || count.get() * 2;
                view! {
                    <p>{count.get()}</p>
                }
            }
            "#,
        );

        assert!(output
            .diagnostics
            .iter()
            .any(|d| d.rule_id == "leptos.signal-get-in-view"));
    }

    #[test]
    fn ignores_rules_inside_comments_and_strings() {
        let output = LeptosDiagnostics::analyze(
            r#"
            // fn Missing() -> impl IntoView { view! { <p>{count.get()}</p> } }
            let text = "create_signal and #[server]";
            "#,
        );

        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn accepts_component_with_attribute() {
        let output = LeptosDiagnostics::analyze(
            r#"
            #[component]
            fn App() -> impl IntoView {
                view! { <p>"ok"</p> }
            }
            "#,
        );

        assert!(!output
            .diagnostics
            .iter()
            .any(|d| d.rule_id == "leptos.missing-component-attribute"));
    }
}
