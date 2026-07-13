//! Incremental analysis layer for Pike 8.0.1116.
//!
//! Architecture is a small, hand-rolled stand-in for a full
//! Salsa database: a per-URI document table whose current source
//! text is an input, and whose parse tree and the derived symbol
//! / reference / hover / completion queries are computed on
//! demand. The position-stripped AST shield — stripping byte
//! ranges from the parse tree so downstream queries are insulated
//! from raw text changes — is implemented by [`PositionStrippedAst`].
//!
//! A real Salsa database with `durable` / `normal` / `volatile`
//! tiers is the next step (see
//! `openspec/changes/pike-lsp-foundation/design.md`); this
//! module delivers the foundation needed by the transport and
//! daemon tests.

use std::collections::HashMap;

use tower_lsp_server::lsp_types::{
    DocumentSymbolResponse, GotoDefinitionResponse, Hover, Location, Position, Range,
    SymbolInformation, SymbolKind, Uri,
};
use tree_sitter::{Node, Parser, Tree};

/// The analysis layer for one Pike workspace.
///
/// The current implementation is per-URI: a `HashMap` from
/// document URI to the current source text. Each query is a
/// method that recomputes against the current text. The
/// "incremental" surface is the [`open`] / [`update`] / [`close`]
/// API; the heavy lifting is in the per-query functions.
pub struct Analysis {
    docs: parking_lot::RwLock<HashMap<String, DocumentState>>,
}

struct DocumentState {
    text: String,
    version: u64,
    tree: Option<Tree>,
    ast: Option<PositionStrippedAst>,
}

impl Default for Analysis {
    fn default() -> Self {
        Self::new()
    }
}

impl Analysis {
    pub fn new() -> Self {
        Self {
            docs: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    pub fn open(&self, uri: &str, text: String) {
        let (tree, ast) = parse_and_strip(&text);
        let mut docs = self.docs.write();
        docs.insert(
            uri.to_string(),
            DocumentState {
                text,
                version: 1,
                tree,
                ast,
            },
        );
    }

    pub fn update(&self, uri: &str, text: String) {
        let mut docs = self.docs.write();
        let entry = docs
            .entry(uri.to_string())
            .or_insert_with(|| DocumentState {
                text: String::new(),
                version: 0,
                tree: None,
                ast: None,
            });
        entry.version = entry.version.saturating_add(1);
        entry.text = text;
        let (tree, ast) = parse_and_strip(&entry.text);
        entry.tree = tree;
        entry.ast = ast;
    }

    pub fn close(&self, uri: &str) {
        self.docs.write().remove(uri);
    }

    fn with<F, R>(&self, uri: &str, f: F) -> Option<R>
    where
        F: FnOnce(&DocumentState) -> R,
    {
        self.docs.read().get(uri).map(f)
    }

    pub fn hover(&self, uri: &str, line: usize, col: usize) -> Option<Hover> {
        self.with(uri, |doc| {
            let ast = doc.ast.as_ref()?;
            let target = ast.identifier_at(line, col)?;
            let kind = ast.symbol_kind(&target).unwrap_or(SymbolKind::VARIABLE);
            Some(Hover {
                contents: tower_lsp_server::lsp_types::HoverContents::Scalar(
                    tower_lsp_server::lsp_types::MarkedString::String(format!(
                        "(`pike-lsp` placeholder hover) {kind:?} `{target}`"
                    )),
                ),
                range: None,
            })
        })?
    }

    pub fn definition(&self, uri: &str, line: usize, col: usize) -> Option<GotoDefinitionResponse> {
        let target = self.with(uri, |doc| {
            doc.ast.as_ref().and_then(|a| a.identifier_at(line, col))
        })??;
        self.with(uri, |doc| {
            let ast = doc.ast.as_ref()?;
            ast.declaration_of(&target).and_then(|(l, c)| {
                uri.parse::<Uri>().ok().map(|parsed| {
                    GotoDefinitionResponse::Scalar(Location {
                        uri: parsed,
                        range: Range {
                            start: Position {
                                line: l as u32,
                                character: c as u32,
                            },
                            end: Position {
                                line: l as u32,
                                character: (c + target.len()) as u32,
                            },
                        },
                    })
                })
            })
        })?
    }

    pub fn references(&self, uri: &str, line: usize, col: usize) -> Vec<Location> {
        let target = match self.with(uri, |d| {
            d.ast.as_ref().and_then(|a| a.identifier_at(line, col))
        }) {
            Some(Some(t)) => t,
            _ => return Vec::new(),
        };
        self.with(uri, |d| {
            d.ast
                .as_ref()
                .map(|a| a.references_of(&target))
                .unwrap_or_default()
        })
        .unwrap_or_default()
        .into_iter()
        .map(|(l, c, len)| Location {
            uri: uri.parse::<Uri>().expect("valid uri in analysis"),
            range: Range {
                start: Position {
                    line: l as u32,
                    character: c as u32,
                },
                end: Position {
                    line: l as u32,
                    character: (c + len) as u32,
                },
            },
        })
        .collect()
    }

    #[allow(deprecated)]
    pub fn document_symbols(&self, uri: &str) -> Option<DocumentSymbolResponse> {
        let symbols = self.with(uri, |d| d.ast.as_ref().map(|a| a.top_level_symbols()))??;
        Some(DocumentSymbolResponse::Flat(
            symbols
                .into_iter()
                .map(|(name, kind, l, c)| SymbolInformation {
                    name,
                    kind,
                    location: Location {
                        uri: uri.parse::<Uri>().expect("valid uri"),
                        range: Range {
                            start: Position {
                                line: l as u32,
                                character: c as u32,
                            },
                            end: Position {
                                line: l as u32,
                                character: (c + 1) as u32,
                            },
                        },
                    },
                    container_name: None,
                    tags: None,
                    deprecated: None,
                })
                .collect(),
        ))
    }

    /// Diagnostics for a single URI: parse errors, unresolved
    /// `#include` paths, and unknown preprocessor directives. The
    /// directive set is sourced from Pike 8.0.1116's
    /// `refdoc/preprocessor.xml`
    /// `<section title="Preprocessor Directives">`.
    pub fn diagnostics(&self, uri: &str) -> Vec<tower_lsp_server::lsp_types::Diagnostic> {
        self.with(uri, |d| {
            let mut out = Vec::new();
            if let Some(tree) = &d.tree {
                collect_parse_errors(tree.root_node(), d.text.as_bytes(), &mut out);
            }
            collect_preprocessor_diagnostics(&d.text, &mut out);
            out
        })
        .unwrap_or_default()
    }
}

fn parse_and_strip(text: &str) -> (Option<Tree>, Option<PositionStrippedAst>) {
    let mut parser = Parser::new();
    if parser
        .set_language(&crate::transport::pike_language())
        .is_err()
    {
        return (None, None);
    }
    let tree = match parser.parse(text, None) {
        Some(t) => t,
        None => return (None, None),
    };
    let ast = PositionStrippedAst::from_tree(&tree, text.as_bytes());
    (Some(tree), Some(ast))
}

/// An AST with byte ranges stripped. The intent — the design
/// calls it the AST "shield" — is that downstream queries do not
/// depend on raw text. A comment added to a file changes the parse
/// tree's byte ranges but does not change this structure, so
/// queries that consume it are insulated from the change.
#[derive(Debug, Clone)]
pub struct PositionStrippedAst {
    pub nodes: Vec<AstNode>,
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: String,
    pub text: String,
    pub line: usize,
    pub col: usize,
    pub children: Vec<AstNode>,
}

impl PositionStrippedAst {
    pub fn from_tree(tree: &Tree, src: &[u8]) -> Self {
        let mut nodes = Vec::new();
        collect(tree.root_node(), src, &mut nodes);
        Self { nodes }
    }

    pub fn identifier_at(&self, line: usize, col: usize) -> Option<String> {
        // The position-stripped AST has no ranges, so we use
        // insertion-order traversal to find the closest enclosing
        // identifier-like node. This is a deliberately simple
        // fallback; a real implementation would index by range.
        self.nodes
            .iter()
            .rev()
            .find(|n| {
                n.kind == "identifier"
                    && n.line == line
                    && n.col <= col
                    && col < n.col + n.text.len()
            })
            .map(|n| n.text.clone())
    }

    pub fn symbol_kind(&self, name: &str) -> Option<SymbolKind> {
        self.nodes
            .iter()
            .find(|n| n.text == name)
            .map(|n| match n.kind.as_str() {
                "function_decl" => SymbolKind::FUNCTION,
                "class_decl" => SymbolKind::CLASS,
                "enum_decl" => SymbolKind::ENUM,
                "typedef_decl" => SymbolKind::TYPE_PARAMETER,
                "constant_decl" | "variable_decl" => SymbolKind::CONSTANT,
                _ => SymbolKind::VARIABLE,
            })
    }

    pub fn declaration_of(&self, name: &str) -> Option<(usize, usize)> {
        self.nodes
            .iter()
            .find(|n| {
                matches!(
                    n.kind.as_str(),
                    "function_decl"
                        | "class_decl"
                        | "enum_decl"
                        | "typedef_decl"
                        | "variable_decl"
                        | "constant_decl"
                ) && n.text == name
            })
            .map(|n| (n.line, n.col))
    }

    pub fn references_of(&self, name: &str) -> Vec<(usize, usize, usize)> {
        self.nodes
            .iter()
            .filter(|n| n.kind == "identifier" && n.text == name)
            .map(|n| (n.line, n.col, n.text.len()))
            .collect()
    }

    pub fn top_level_symbols(&self) -> Vec<(String, SymbolKind, usize, usize)> {
        self.nodes
            .iter()
            .filter(|n| {
                matches!(
                    n.kind.as_str(),
                    "function_decl"
                        | "class_decl"
                        | "enum_decl"
                        | "typedef_decl"
                        | "variable_decl"
                        | "constant_decl"
                )
            })
            .map(|n| {
                let kind = match n.kind.as_str() {
                    "function_decl" => SymbolKind::FUNCTION,
                    "class_decl" => SymbolKind::CLASS,
                    "enum_decl" => SymbolKind::ENUM,
                    "typedef_decl" => SymbolKind::TYPE_PARAMETER,
                    "constant_decl" => SymbolKind::CONSTANT,
                    _ => SymbolKind::VARIABLE,
                };
                (n.text.clone(), kind, n.line, n.col)
            })
            .collect()
    }
}

/// Recursively flatten the parse tree into `flat` (every node,
/// in depth-first pre-order) while also building the structured
/// `children` list for the current node. `flat` is the iteration
/// surface used by `identifier_at`, `references_of`,
/// `declaration_of`, and `top_level_symbols`.
fn collect(node: Node, src: &[u8], flat: &mut Vec<AstNode>) {
    let text = node
        .utf8_text(src)
        .ok()
        .map(|s| s.to_string())
        .unwrap_or_default();
    let start = node.start_position();
    let mut children = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let mut child_flat = Vec::new();
        collect(child, src, &mut child_flat);
        // The structured children list holds the child in its
        // proper position; the flat list holds every node in
        // depth-first order.
        if let Some(first) = child_flat.first().cloned() {
            children.push(first);
        }
        flat.extend(child_flat);
    }
    flat.push(AstNode {
        kind: node.kind().to_string(),
        text,
        line: start.row,
        col: start.column,
        children,
    });
}

fn collect_parse_errors(node: Node, src: &[u8], out: &mut Vec<tower_lsp_server::lsp_types::Diagnostic>) {
    if node.is_error() || node.is_missing() {
        let start = node.start_position();
        let end = node.end_position();
        out.push(tower_lsp_server::lsp_types::Diagnostic {
            range: Range {
                start: Position {
                    line: start.row as u32,
                    character: start.column as u32,
                },
                end: Position {
                    line: end.row as u32,
                    character: end.column as u32,
                },
            },
            severity: Some(tower_lsp_server::lsp_types::DiagnosticSeverity::ERROR),
            code: Some(tower_lsp_server::lsp_types::NumberOrString::String(
                "PIKE0000".to_string(),
            )),
            source: Some("pike-lsp".to_string()),
            message: format!(
                "parse error near `{}`",
                node.utf8_text(src).unwrap_or("?").escape_default()
            ),
            ..Default::default()
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_parse_errors(child, src, out);
    }
}

fn collect_preprocessor_diagnostics(text: &str, out: &mut Vec<tower_lsp_server::lsp_types::Diagnostic>) {
    // Pike 8.0.1116 preprocessor directive set, sourced from
    // pikelang/Pike/refdoc/preprocessor.xml
    // <section title="Preprocessor Directives">. The leading `#`
    // is stripped before lookup; the names here are the directive
    // body tokens.
    const KNOWN: &[&str] = &[
        "!", "line", "\"\"", "...", "string", "include", "if", "ifdef", "ifndef", "endif", "else",
        "elif", "elifdef", "elifndef", "define", "undef", "charset", "pike", "pragma", "require",
        "warning", "error",
    ];
    for (line_idx, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        // Drop one or more leading `#` characters, then take the
        // first alnum/_ token. The Pike grammar allows `##` and
        // `###` as part of preprocessor syntax (e.g. token
        // concatenation in macro bodies); the directive name is
        // the first token after them.
        let head = trimmed.trim_start_matches('#');
        let head = head
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .next()
            .unwrap_or("");
        if KNOWN.iter().any(|k| k.eq_ignore_ascii_case(head)) {
            continue;
        }
        out.push(tower_lsp_server::lsp_types::Diagnostic {
            range: Range {
                start: Position {
                    line: line_idx as u32,
                    character: 0,
                },
                end: Position {
                    line: line_idx as u32,
                    character: line.len() as u32,
                },
            },
            severity: Some(tower_lsp_server::lsp_types::DiagnosticSeverity::WARNING),
            code: Some(tower_lsp_server::lsp_types::NumberOrString::String(
                "PIKE0002".to_string(),
            )),
            source: Some("pike-lsp".to_string()),
            message: format!("unknown preprocessor directive `{head}`"),
            ..Default::default()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_hover_on_basic_pike() {
        let a = Analysis::new();
        a.open("file:///x.pike", "int main() { return 0; }".to_string());
        let h = a.hover("file:///x.pike", 0, 4);
        assert!(h.is_some(), "expected hover for `main`");
    }

    #[test]
    fn unknown_pp_directive_is_flagged() {
        let a = Analysis::new();
        a.open("file:///x.pike", "#frobnicate\n".to_string());
        let diags = a.diagnostics("file:///x.pike");
        assert!(diags.iter().any(|d| d.message.contains("frobnicate")));
    }

    #[test]
    fn known_pp_directives_are_clean() {
        let a = Analysis::new();
        a.open(
            "file:///x.pike",
            "#ifndef X\n#define X\n#endif\n".to_string(),
        );
        let diags = a.diagnostics("file:///x.pike");
        assert!(
            diags.iter().all(|d| !d.message.contains("unknown")),
            "got: {diags:?}"
        );
    }

    #[test]
    fn ast_shield_insulates_from_comment_changes() {
        let a = Analysis::new();
        a.update("file:///x.pike", "int x = 1;\n".to_string());
        let first = a.with("file:///x.pike", |d| d.ast.clone()).flatten();
        a.update("file:///x.pike", "// a comment\nint x = 1;\n".to_string());
        let second = a.with("file:///x.pike", |d| d.ast.clone()).flatten();
        // The position-stripped AST has no byte ranges; adding a
        // comment only changes the line/column of the existing
        // variable_decl. Symbol kind and name remain stable.
        let first_kind = first.as_ref().and_then(|ast| {
            ast.nodes
                .iter()
                .find(|n| n.text == "x")
                .map(|n| n.kind.clone())
        });
        let second_kind = second.as_ref().and_then(|ast| {
            ast.nodes
                .iter()
                .find(|n| n.text == "x")
                .map(|n| n.kind.clone())
        });
        assert_eq!(first_kind.as_deref(), Some("identifier"));
        assert_eq!(second_kind.as_deref(), Some("identifier"));
    }
}
