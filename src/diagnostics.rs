//! Leptos code diagnostics.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
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

impl Diagnostic {
    fn new(rule: RuleMetadata, span: SourceSpan) -> Self {
        debug_assert!(
            rule.allows_error_high
                || rule.severity != Severity::Error
                || rule.confidence != Confidence::High,
            "{} emits Severity::Error + Confidence::High without metadata approval",
            rule.rule_id
        );

        Self {
            rule_id: rule.rule_id,
            severity: rule.severity,
            message: rule.message,
            span,
            confidence: rule.confidence,
            suggested_fix: rule.suggested_fix,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RuleMetadata {
    rule_id: &'static str,
    severity: Severity,
    confidence: Confidence,
    message: &'static str,
    suggested_fix: Option<&'static str>,
    /// Only rules backed by structural evidence should set this. Speculative
    /// substring rules must not emit the strongest diagnostic combination.
    allows_error_high: bool,
}

const SIGNAL_GET_IN_VIEW: RuleMetadata = RuleMetadata {
    rule_id: "leptos.signal-get-in-view",
    severity: Severity::Warning,
    confidence: Confidence::Medium,
    message: "Reactive signal reads inside views should be wrapped in `move ||`.",
    suggested_fix: Some("Use `{move || value.get()}` for reactive view updates."),
    allows_error_high: false,
};

const SIGNAL_DESTRUCTURING: RuleMetadata = RuleMetadata {
    rule_id: "leptos.signal-destructuring",
    severity: Severity::Warning,
    confidence: Confidence::Medium,
    message: "Signals are clearer when destructured into getter and setter bindings.",
    suggested_fix: Some("Use `let (getter, setter) = signal(value);`."),
    allows_error_high: false,
};

const MISSING_COMPONENT_ATTRIBUTE: RuleMetadata = RuleMetadata {
    rule_id: "leptos.missing-component-attribute",
    severity: Severity::Warning,
    confidence: Confidence::Medium,
    message: "Functions returning `impl IntoView` should be annotated with `#[component]`.",
    suggested_fix: Some("Add `#[component]` immediately above the component function."),
    allows_error_high: false,
};

const SERVER_FN_ERROR: RuleMetadata = RuleMetadata {
    rule_id: "leptos.server-fn-error",
    severity: Severity::Info,
    confidence: Confidence::Medium,
    message: "Server functions should return `Result<T, ServerFnError>`.",
    suggested_fix: Some("Return `Result<T, ServerFnError>` from server functions."),
    allows_error_high: false,
};

const SERVER_FN_ASYNC: RuleMetadata = RuleMetadata {
    rule_id: "leptos.server-fn-async",
    severity: Severity::Error,
    confidence: Confidence::High,
    message: "Server functions must be async.",
    suggested_fix: Some("Add `async` to the server function signature."),
    allows_error_high: true,
};

const SERVER_FN_GENERIC: RuleMetadata = RuleMetadata {
    rule_id: "leptos.server-fn-generic",
    severity: Severity::Error,
    confidence: Confidence::High,
    message: "Server functions cannot be generic.",
    suggested_fix: Some(
        "Move generic logic into a private helper and expose concrete server functions.",
    ),
    allows_error_high: true,
};

const SERVER_FN_PREFIX: RuleMetadata = RuleMetadata {
    rule_id: "leptos.server-fn-prefix",
    severity: Severity::Warning,
    confidence: Confidence::Medium,
    message: "Server function prefixes should be absolute paths.",
    suggested_fix: Some("Use a prefix like `\"/api\"`."),
    allows_error_high: false,
};

const SERVER_FN_DUPLICATE_PATH: RuleMetadata = RuleMetadata {
    rule_id: "leptos.server-fn-duplicate-path",
    severity: Severity::Warning,
    confidence: Confidence::Medium,
    message: "Server function endpoint paths must be unique.",
    suggested_fix: Some("Use a unique prefix or endpoint path for each server function."),
    allows_error_high: false,
};

const EXTRACT_STATE: RuleMetadata = RuleMetadata {
    rule_id: "leptos-axum.extract-state",
    severity: Severity::Warning,
    confidence: Confidence::Medium,
    message: "Server functions using Axum State extractors should use `extract_with_state()`.",
    suggested_fix: Some("Use `leptos_axum::extract_with_state(&state).await?`."),
    allows_error_high: false,
};

const EXTRACT_BODY: RuleMetadata = RuleMetadata {
    rule_id: "leptos-axum.extract-body",
    severity: Severity::Warning,
    confidence: Confidence::Medium,
    message: "Server functions should not use body-consuming Axum extractors with `extract()`.",
    suggested_fix: Some("Pass body data as server function arguments instead."),
    allows_error_high: false,
};

const DEPRECATED_CREATE_SIGNAL: RuleMetadata = RuleMetadata {
    rule_id: "leptos.deprecated-create-signal",
    severity: Severity::Info,
    confidence: Confidence::Medium,
    message: "In Leptos 0.8+, prefer `signal()` over `create_signal()`.",
    suggested_fix: Some("Use `signal(value)` instead of `create_signal(value)`."),
    allows_error_high: false,
};

#[cfg(test)]
const ALL_RULES: &[RuleMetadata] = &[
    SIGNAL_GET_IN_VIEW,
    SIGNAL_DESTRUCTURING,
    MISSING_COMPONENT_ATTRIBUTE,
    SERVER_FN_ERROR,
    SERVER_FN_ASYNC,
    SERVER_FN_GENERIC,
    SERVER_FN_PREFIX,
    SERVER_FN_DUPLICATE_PATH,
    EXTRACT_STATE,
    EXTRACT_BODY,
    DEPRECATED_CREATE_SIGNAL,
];

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
        let structural = strip_comments_preserve_strings(code);
        let server_functions = find_server_functions(&structural);
        let mut diagnostics = Vec::new();

        diagnostics.extend(detect_signal_get_in_view(&searchable));
        diagnostics.extend(detect_signal_destructuring(&searchable));
        diagnostics.extend(detect_missing_component_attribute(&searchable));
        diagnostics.extend(detect_server_fn_error(&server_functions));
        diagnostics.extend(detect_server_fn_async(&server_functions));
        diagnostics.extend(detect_server_fn_generic(&server_functions));
        diagnostics.extend(detect_invalid_server_fn_prefix(&server_functions));
        diagnostics.extend(detect_duplicate_server_fn_paths(&server_functions));
        diagnostics.extend(detect_state_extractor_without_state_helper(&structural));
        diagnostics.extend(detect_body_extractor_in_server_function(&structural));
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServerFunctionCandidate {
    line: usize,
    column: usize,
    attribute: String,
    signature: String,
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

            Some(Diagnostic::new(
                SIGNAL_GET_IN_VIEW,
                SourceSpan {
                    line: index + 1,
                    column: column + 1,
                },
            ))
        })
        .collect()
}

fn detect_signal_destructuring(searchable: &str) -> Vec<Diagnostic> {
    searchable
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let column = line.find("let signal =")?;
            Some(Diagnostic::new(
                SIGNAL_DESTRUCTURING,
                SourceSpan {
                    line: index + 1,
                    column: column + 1,
                },
            ))
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
            let function_name = function_name(line)?;
            if !function_name
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_uppercase())
            {
                return None;
            }
            if has_component_attribute_before(&lines, index) {
                return None;
            }

            Some(Diagnostic::new(
                MISSING_COMPONENT_ATTRIBUTE,
                SourceSpan {
                    line: index + 1,
                    column: column + 1,
                },
            ))
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

fn detect_server_fn_error(server_functions: &[ServerFunctionCandidate]) -> Vec<Diagnostic> {
    server_functions
        .iter()
        .filter_map(|candidate| {
            if candidate.signature.contains("ServerFnError") {
                return None;
            }

            Some(Diagnostic::new(
                SERVER_FN_ERROR,
                SourceSpan {
                    line: candidate.line,
                    column: candidate.column,
                },
            ))
        })
        .collect()
}

fn detect_server_fn_async(server_functions: &[ServerFunctionCandidate]) -> Vec<Diagnostic> {
    server_functions
        .iter()
        .filter_map(|candidate| {
            if candidate.signature.contains("async fn") {
                return None;
            }

            Some(Diagnostic::new(
                SERVER_FN_ASYNC,
                SourceSpan {
                    line: candidate.line,
                    column: candidate.column,
                },
            ))
        })
        .collect()
}

fn detect_server_fn_generic(server_functions: &[ServerFunctionCandidate]) -> Vec<Diagnostic> {
    server_functions
        .iter()
        .filter_map(|candidate| {
            if !signature_has_generic_fn(&candidate.signature) {
                return None;
            }

            Some(Diagnostic::new(
                SERVER_FN_GENERIC,
                SourceSpan {
                    line: candidate.line,
                    column: candidate.column,
                },
            ))
        })
        .collect()
}

fn detect_invalid_server_fn_prefix(
    server_functions: &[ServerFunctionCandidate],
) -> Vec<Diagnostic> {
    server_functions
        .iter()
        .filter_map(|candidate| {
            let prefix = attribute_value(&candidate.attribute, "prefix")?;
            if prefix.starts_with('/') {
                return None;
            }

            Some(Diagnostic::new(
                SERVER_FN_PREFIX,
                SourceSpan {
                    line: candidate.line,
                    column: candidate.column,
                },
            ))
        })
        .collect()
}

fn detect_duplicate_server_fn_paths(
    server_functions: &[ServerFunctionCandidate],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: Vec<(String, usize, usize)> = Vec::new();

    for candidate in server_functions {
        let Some(endpoint) = attribute_value(&candidate.attribute, "endpoint") else {
            continue;
        };
        let prefix = attribute_value(&candidate.attribute, "prefix").unwrap_or("/api");
        let path = format!("{prefix}/{endpoint}");

        if seen.iter().any(|(seen_path, _, _)| seen_path == &path) {
            diagnostics.push(Diagnostic::new(
                SERVER_FN_DUPLICATE_PATH,
                SourceSpan {
                    line: candidate.line,
                    column: candidate.column,
                },
            ));
        } else {
            seen.push((path, candidate.line, candidate.column));
        }
    }

    diagnostics
}

fn detect_state_extractor_without_state_helper(structural: &str) -> Vec<Diagnostic> {
    if !structural.contains("#[server")
        || !structural.contains("State<")
        || !structural.contains("extract().await")
        || structural.contains("extract_with_state")
    {
        return Vec::new();
    }

    vec![Diagnostic::new(
        EXTRACT_STATE,
        find_span(structural, "extract().await"),
    )]
}

fn detect_body_extractor_in_server_function(structural: &str) -> Vec<Diagnostic> {
    if !structural.contains("#[server") || !structural.contains("extract().await") {
        return Vec::new();
    }
    if !(structural.contains("Json<")
        || structural.contains("Form<")
        || structural.contains("Multipart"))
    {
        return Vec::new();
    }

    vec![Diagnostic::new(
        EXTRACT_BODY,
        find_span(structural, "extract().await"),
    )]
}

fn detect_deprecated_create_signal(searchable: &str) -> Vec<Diagnostic> {
    searchable
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let column = line.find("create_signal")?;
            Some(Diagnostic::new(
                DEPRECATED_CREATE_SIGNAL,
                SourceSpan {
                    line: index + 1,
                    column: column + 1,
                },
            ))
        })
        .collect()
}

fn function_name(line: &str) -> Option<&str> {
    let after_fn = line.split_once("fn ")?.1;
    let name_len = after_fn
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .map(char::len_utf8)
        .sum();
    (name_len > 0).then(|| &after_fn[..name_len])
}

fn find_server_functions(code: &str) -> Vec<ServerFunctionCandidate> {
    let lines: Vec<&str> = code.lines().collect();
    let mut candidates = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let Some(column) = line.find("#[server") else {
            index += 1;
            continue;
        };
        if !line[..column].trim().is_empty() {
            index += 1;
            continue;
        }

        let start_index = index;
        let mut attribute_lines = vec![line.trim().to_string()];
        while !attribute_lines.join(" ").contains(']') && index + 1 < lines.len() {
            index += 1;
            attribute_lines.push(lines[index].trim().to_string());
        }

        let mut signature_lines = Vec::new();
        let mut signature_index = index + 1;
        while signature_index < lines.len() && signature_lines.len() < 8 {
            let signature_line = lines[signature_index].trim();
            if signature_line.is_empty() || signature_line.starts_with("#[") {
                signature_index += 1;
                continue;
            }
            signature_lines.push(signature_line.to_string());
            if signature_line.contains('{') || signature_line.ends_with(';') {
                break;
            }
            signature_index += 1;
        }

        candidates.push(ServerFunctionCandidate {
            line: start_index + 1,
            column: column + 1,
            attribute: attribute_lines.join(" "),
            signature: signature_lines.join(" "),
        });

        index = signature_index.max(index + 1);
    }

    candidates
}

fn signature_has_generic_fn(signature: &str) -> bool {
    let Some(after_fn) = signature.split_once("fn ").map(|(_, value)| value) else {
        return false;
    };
    let name_len: usize = after_fn
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .map(char::len_utf8)
        .sum();

    after_fn[name_len..].trim_start().starts_with('<')
}

fn attribute_value<'a>(attribute: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("{key} = \"");
    let after_marker = attribute.split_once(&marker)?.1;
    let end = after_marker.find('"')?;
    Some(&after_marker[..end])
}

fn find_span(text: &str, needle: &str) -> SourceSpan {
    let mut line = 1;
    let mut column = 1;
    for current_line in text.lines() {
        if let Some(index) = current_line.find(needle) {
            return SourceSpan {
                line,
                column: index + 1,
            };
        }
        line += 1;
        column = 1;
    }

    SourceSpan { line, column }
}

fn strip_comments_preserve_strings(code: &str) -> String {
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
            output.push(ch);
            if ch == '"' && !escaped {
                in_string = false;
            }
            escaped = ch == '\\' && !escaped;
            if ch != '\\' {
                escaped = false;
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
        } else {
            if ch == '"' {
                in_string = true;
                escaped = false;
            }
            output.push(ch);
        }
    }

    output
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

    fn assert_diagnostic(
        output: &DiagnosticsOutput,
        rule_id: &'static str,
        severity: Severity,
        message: &'static str,
        span: SourceSpan,
        confidence: Confidence,
    ) {
        let diagnostic = output
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.rule_id == rule_id)
            .unwrap_or_else(|| panic!("missing diagnostic {rule_id}: {output:#?}"));

        assert_eq!(diagnostic.rule_id, rule_id);
        assert_eq!(diagnostic.severity, severity);
        assert_eq!(diagnostic.message, message);
        assert_eq!(diagnostic.span, span);
        assert_eq!(diagnostic.confidence, confidence);
    }

    fn assert_no_diagnostic(output: &DiagnosticsOutput, rule_id: &'static str) {
        assert!(
            !output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == rule_id),
            "unexpected diagnostic {rule_id}: {output:#?}"
        );
    }

    fn expected_span(code: &str, needle: &str) -> SourceSpan {
        for (line_index, line) in code.lines().enumerate() {
            if let Some(column_index) = line.find(needle) {
                return SourceSpan {
                    line: line_index + 1,
                    column: column_index + 1,
                };
            }
        }

        panic!("needle {needle:?} not found in test code");
    }

    #[test]
    fn rule_metadata_documents_high_confidence_error_emitters() {
        let high_confidence_error_rules: Vec<&'static str> = ALL_RULES
            .iter()
            .filter(|rule| rule.severity == Severity::Error && rule.confidence == Confidence::High)
            .map(|rule| rule.rule_id)
            .collect();

        assert_eq!(
            high_confidence_error_rules,
            vec!["leptos.server-fn-async", "leptos.server-fn-generic"]
        );
        assert!(
            ALL_RULES
                .iter()
                .filter(
                    |rule| rule.severity == Severity::Error && rule.confidence == Confidence::High
                )
                .all(|rule| rule.allows_error_high)
        );
    }

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

        assert!(
            output
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "leptos.signal-get-in-view")
        );
    }

    #[test]
    fn characterizes_signal_get_in_view_diagnostic_and_move_closure_exemption() {
        let code = r#"#[component]
fn Counter() -> impl IntoView {
    view! { <p>{count.get()}</p> }
}
"#;
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos.signal-get-in-view",
            Severity::Warning,
            "Reactive signal reads inside views should be wrapped in `move ||`.",
            expected_span(code, ".get()"),
            Confidence::Medium,
        );

        let closure_code = r#"#[component]
fn Counter() -> impl IntoView {
    view! { <p>{move || count.get()}</p> }
}
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(closure_code),
            "leptos.signal-get-in-view",
        );
    }

    #[test]
    fn render_diagnostics_preserves_downgraded_rule_text() {
        let code = r#"#[component]
fn Counter() -> impl IntoView {
    view! { <p>{count.get()}</p> }
}
"#;
        let output = LeptosDiagnostics::analyze(code);
        let rendered = render_diagnostics(&output);

        assert!(rendered.contains("WARNING [leptos.signal-get-in-view]"));
        assert!(
            rendered.contains("Reactive signal reads inside views should be wrapped in `move ||`.")
        );
    }

    #[test]
    fn characterizes_signal_destructuring_diagnostic_and_tuple_binding_exemption() {
        let code = "let signal = signal(0);\n";
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos.signal-destructuring",
            Severity::Warning,
            "Signals are clearer when destructured into getter and setter bindings.",
            expected_span(code, "let signal ="),
            Confidence::Medium,
        );

        assert_no_diagnostic(
            &LeptosDiagnostics::analyze("let (count, set_count) = signal(0);\n"),
            "leptos.signal-destructuring",
        );
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

        assert!(
            !output
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "leptos.missing-component-attribute")
        );
    }

    #[test]
    fn does_not_require_component_attribute_for_lowercase_view_helpers() {
        let output = LeptosDiagnostics::analyze(
            r#"
            fn render_label() -> impl IntoView {
                view! { <span>"label"</span> }
            }
            "#,
        );

        assert!(
            !output
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "leptos.missing-component-attribute")
        );
    }

    #[test]
    fn characterizes_missing_component_attribute_diagnostic_and_exemptions() {
        let code = r#"fn Missing() -> impl IntoView {
    view! { <p>"missing"</p> }
}
"#;
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos.missing-component-attribute",
            Severity::Warning,
            "Functions returning `impl IntoView` should be annotated with `#[component]`.",
            expected_span(code, "-> impl IntoView"),
            Confidence::Medium,
        );

        let attributed = r#"#[component]
fn Present() -> impl IntoView {
    view! { <p>"present"</p> }
}
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(attributed),
            "leptos.missing-component-attribute",
        );

        let lowercase = r#"fn helper() -> impl IntoView {
    view! { <p>"helper"</p> }
}
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(lowercase),
            "leptos.missing-component-attribute",
        );
    }

    #[test]
    fn reports_server_functions_that_are_not_async_or_are_generic() {
        let output = LeptosDiagnostics::analyze(
            r#"
            #[server(GenericThing)]
            pub fn generic_thing<T>(value: String) -> Result<String, ServerFnError> {
                Ok(value)
            }
            "#,
        );

        assert!(
            output
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "leptos.server-fn-async")
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "leptos.server-fn-generic")
        );
    }

    #[test]
    fn characterizes_server_function_return_error_diagnostic_and_server_fn_error_exemption() {
        let code = r#"#[server(Load)]
pub async fn load() -> Result<String, AppError> {
    Ok(String::new())
}
"#;
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos.server-fn-error",
            Severity::Info,
            "Server functions should return `Result<T, ServerFnError>`.",
            expected_span(code, "#[server"),
            Confidence::Medium,
        );

        let ok_code = r#"#[server(Load)]
pub async fn load() -> Result<String, ServerFnError> {
    Ok(String::new())
}
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(ok_code),
            "leptos.server-fn-error",
        );
    }

    #[test]
    fn characterizes_server_function_async_diagnostic_and_async_exemption() {
        let code = r#"#[server(Save)]
pub fn save() -> Result<(), ServerFnError> {
    Ok(())
}
"#;
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos.server-fn-async",
            Severity::Error,
            "Server functions must be async.",
            expected_span(code, "#[server"),
            Confidence::High,
        );

        let async_code = r#"#[server(Save)]
pub async fn save() -> Result<(), ServerFnError> {
    Ok(())
}
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(async_code),
            "leptos.server-fn-async",
        );
    }

    #[test]
    fn characterizes_server_function_generic_diagnostic_and_concrete_exemption() {
        let code = r#"#[server(Save)]
pub async fn save<T>(value: T) -> Result<(), ServerFnError> {
    Ok(())
}
"#;
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos.server-fn-generic",
            Severity::Error,
            "Server functions cannot be generic.",
            expected_span(code, "#[server"),
            Confidence::High,
        );

        let concrete_code = r#"#[server(Save)]
pub async fn save(value: String) -> Result<(), ServerFnError> {
    Ok(())
}
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(concrete_code),
            "leptos.server-fn-generic",
        );
    }

    #[test]
    fn reports_invalid_and_duplicate_server_function_paths() {
        let output = LeptosDiagnostics::analyze(
            r#"
            #[server(One, prefix = "api", endpoint = "save")]
            pub async fn one() -> Result<(), ServerFnError> { Ok(()) }

            #[server(Two, prefix = "api", endpoint = "save")]
            pub async fn two() -> Result<(), ServerFnError> { Ok(()) }
            "#,
        );

        assert!(
            output
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "leptos.server-fn-prefix")
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "leptos.server-fn-duplicate-path")
        );
    }

    #[test]
    fn characterizes_server_function_prefix_diagnostic_and_absolute_prefix_exemption() {
        let code = r#"#[server(Save, prefix = "api", endpoint = "save")]
pub async fn save() -> Result<(), ServerFnError> { Ok(()) }
"#;
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos.server-fn-prefix",
            Severity::Warning,
            "Server function prefixes should be absolute paths.",
            expected_span(code, "#[server"),
            Confidence::Medium,
        );

        let absolute_code = r#"#[server(Save, prefix = "/api", endpoint = "save")]
pub async fn save() -> Result<(), ServerFnError> { Ok(()) }
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(absolute_code),
            "leptos.server-fn-prefix",
        );
    }

    #[test]
    fn characterizes_duplicate_server_function_path_diagnostic_and_unique_path_exemption() {
        let code = r#"#[server(One, prefix = "/api", endpoint = "save")]
pub async fn one() -> Result<(), ServerFnError> { Ok(()) }

#[server(Two, prefix = "/api", endpoint = "save")]
pub async fn two() -> Result<(), ServerFnError> { Ok(()) }
"#;
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos.server-fn-duplicate-path",
            Severity::Warning,
            "Server function endpoint paths must be unique.",
            SourceSpan { line: 4, column: 1 },
            Confidence::Medium,
        );

        let unique_code = r#"#[server(One, prefix = "/api", endpoint = "save-one")]
pub async fn one() -> Result<(), ServerFnError> { Ok(()) }

#[server(Two, prefix = "/api", endpoint = "save-two")]
pub async fn two() -> Result<(), ServerFnError> { Ok(()) }
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(unique_code),
            "leptos.server-fn-duplicate-path",
        );
    }

    #[test]
    fn reports_axum_state_and_body_extractor_misuse_in_server_functions() {
        let output = LeptosDiagnostics::analyze(
            r#"
            use axum::{extract::State, Json};

            #[server(Save)]
            pub async fn save() -> Result<(), ServerFnError> {
                let State(state): State<AppState> = leptos_axum::extract().await?;
                let Json(body): Json<SaveBody> = leptos_axum::extract().await?;
                Ok(())
            }
            "#,
        );

        assert!(
            output
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "leptos-axum.extract-state")
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "leptos-axum.extract-body")
        );
    }

    #[test]
    fn characterizes_axum_state_extractor_diagnostic_and_extract_with_state_exemption() {
        let code = r#"use axum::extract::State;

#[server(Load)]
pub async fn load() -> Result<(), ServerFnError> {
    let State(state): State<AppState> = leptos_axum::extract().await?;
    Ok(())
}
"#;
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos-axum.extract-state",
            Severity::Warning,
            "Server functions using Axum State extractors should use `extract_with_state()`.",
            expected_span(code, "extract().await"),
            Confidence::Medium,
        );

        let helper_code = r#"use axum::extract::State;

#[server(Load)]
pub async fn load() -> Result<(), ServerFnError> {
    let State(state): State<AppState> = leptos_axum::extract_with_state(&state).await?;
    Ok(())
}
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(helper_code),
            "leptos-axum.extract-state",
        );
    }

    #[test]
    fn characterizes_axum_body_extractor_diagnostic_and_non_body_extractor_exemption() {
        let code = r#"use axum::Json;

#[server(Save)]
pub async fn save() -> Result<(), ServerFnError> {
    let Json(body): Json<SaveBody> = leptos_axum::extract().await?;
    Ok(())
}
"#;
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos-axum.extract-body",
            Severity::Warning,
            "Server functions should not use body-consuming Axum extractors with `extract()`.",
            expected_span(code, "extract().await"),
            Confidence::Medium,
        );

        let non_body_code = r#"use axum::extract::Path;

#[server(Load)]
pub async fn load() -> Result<(), ServerFnError> {
    let Path(id): Path<String> = leptos_axum::extract().await?;
    Ok(())
}
"#;
        assert_no_diagnostic(
            &LeptosDiagnostics::analyze(non_body_code),
            "leptos-axum.extract-body",
        );
    }

    #[test]
    fn characterizes_deprecated_create_signal_diagnostic_and_signal_exemption() {
        let code = "let (count, set_count) = create_signal(0);\n";
        let output = LeptosDiagnostics::analyze(code);

        assert_diagnostic(
            &output,
            "leptos.deprecated-create-signal",
            Severity::Info,
            "In Leptos 0.8+, prefer `signal()` over `create_signal()`.",
            expected_span(code, "create_signal"),
            Confidence::Medium,
        );

        assert_no_diagnostic(
            &LeptosDiagnostics::analyze("let (count, set_count) = signal(0);\n"),
            "leptos.deprecated-create-signal",
        );
    }
}
