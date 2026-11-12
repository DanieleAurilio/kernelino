//! Validates `docs/graph.json`, the machine-readable map of the repository
//! (ADR 0009): its nodes name existing files of the right kind, every mapped
//! file is a node, and its edges link existing nodes with the allowed kinds.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path};

use serde::Deserialize;

const NODE_KINDS: [&str; 5] = ["module", "adr", "context", "doc", "test"];
const EDGE_KINDS: [&str; 5] = [
    "implements",
    "governs",
    "depends_on",
    "tested_by",
    "relates_to",
];
const CONTEXT_PAGES: [&str; 5] = [
    "shell.md",
    "kernel.md",
    "filesystem.md",
    "memory.md",
    "processes.md",
];
/// Mapped files outside the folders listed by `mapped_files`.
const MAPPED_DOCS: [&str; 3] = ["docs/README.md", "docs/CONTEXT.md", "docs/graph.json"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Graph {
    version: u32,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Node {
    id: String,
    kind: String,
    path: String,
    #[allow(dead_code)]
    title: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edge {
    from: String,
    to: String,
    kind: String,
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn load() -> Graph {
    let path = root().join("docs").join("graph.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|err| panic!("{path:?}: {err}"));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("{path:?}: {err}"))
}

fn assert_no_errors(errors: Vec<String>) {
    assert!(
        errors.is_empty(),
        "docs/graph.json:\n- {}",
        errors.join("\n- ")
    );
}

/// The node kind a file must have, from its path: `doc` for any other file.
fn kind_of_path(path: &str) -> &'static str {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        ["src", file] if file.ends_with(".rs") || file.ends_with(".lua") => "module",
        ["docs", "adr", file] if is_adr_file(file) => "adr",
        ["docs", "ddd", file] if CONTEXT_PAGES.contains(file) => "context",
        ["tests", file] if file.ends_with(".rs") => "test",
        _ => "doc",
    }
}

/// `NNNN-<title>.md`.
fn is_adr_file(file: &str) -> bool {
    let bytes = file.as_bytes();
    file.len() > "0000-.md".len()
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && file.ends_with(".md")
}

/// The files of `dir` (not recursive) with one of `extensions`, as paths
/// relative to the repository root.
fn files_in(dir: &str, extensions: &[&str]) -> Vec<String> {
    let full = root().join(dir);
    fs::read_dir(&full)
        .unwrap_or_else(|err| panic!("{full:?}: {err}"))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| extensions.iter().any(|e| ext == *e))
        })
        .map(|path| format!("{dir}/{}", path.file_name().unwrap().to_string_lossy()))
        .collect()
}

/// Every file that must be the path of exactly one node.
fn mapped_files() -> Vec<String> {
    let mut files = files_in("src", &["rs", "lua"]);
    files.extend(files_in("docs/adr", &["md"]));
    files.extend(files_in("docs/ddd", &["md"]));
    files.extend(files_in("tests", &["rs"]));
    files.extend(MAPPED_DOCS.iter().map(|file| file.to_string()));
    files.sort();
    files
}

#[test]
fn version_is_1() {
    let graph = load();
    assert_eq!(graph.version, 1, "docs/graph.json: unsupported version");
}

#[test]
fn ids_are_unique_and_match_their_kind() {
    let graph = load();
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for node in &graph.nodes {
        if !seen.insert(node.id.as_str()) {
            errors.push(format!("duplicate node id `{}`", node.id));
        }
        if !NODE_KINDS.contains(&node.kind.as_str()) {
            errors.push(format!(
                "node `{}`: unknown kind `{}` (expected one of {NODE_KINDS:?})",
                node.id, node.kind
            ));
        }
        let slug_ok = node.id.split_once(':').is_some_and(|(kind, slug)| {
            kind == node.kind
                && !slug.is_empty()
                && slug
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        });
        if !slug_ok {
            errors.push(format!(
                "node `{}`: id must be `{}:<slug>`, slug in [a-z0-9_-]",
                node.id, node.kind
            ));
        }
    }
    assert_no_errors(errors);
}

#[test]
fn paths_exist() {
    let graph = load();
    let mut errors = Vec::new();
    for node in &graph.nodes {
        let relative = Path::new(&node.path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)));
        if !relative {
            errors.push(format!(
                "node `{}`: path `{}` is not relative to the repository root",
                node.id, node.path
            ));
        } else if !root().join(&node.path).is_file() {
            errors.push(format!(
                "node `{}`: path `{}` does not exist",
                node.id, node.path
            ));
        }
    }
    assert_no_errors(errors);
}

#[test]
fn paths_match_their_kind() {
    let graph = load();
    let mut errors = Vec::new();
    for node in &graph.nodes {
        let expected = kind_of_path(&node.path);
        if node.kind != expected {
            errors.push(format!(
                "node `{}`: path `{}` is a `{expected}` path, not a `{}` one",
                node.id, node.path, node.kind
            ));
        }
    }
    assert_no_errors(errors);
}

#[test]
fn every_mapped_file_is_exactly_one_node() {
    let graph = load();
    let mut errors = Vec::new();
    let mut nodes_by_path: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in &graph.nodes {
        nodes_by_path
            .entry(node.path.as_str())
            .or_default()
            .push(node.id.as_str());
    }
    for file in mapped_files() {
        if !nodes_by_path.contains_key(file.as_str()) {
            errors.push(format!("file `{file}` is not the path of any node"));
        }
    }
    let mut shared: Vec<_> = nodes_by_path
        .iter()
        .filter(|(_, ids)| ids.len() > 1)
        .collect();
    shared.sort();
    for (path, ids) in shared {
        errors.push(format!("path `{path}` is shared by nodes {ids:?}"));
    }
    assert_no_errors(errors);
}

#[test]
fn edges_link_existing_nodes_with_allowed_kinds() {
    let graph = load();
    let kinds: HashMap<&str, &str> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.kind.as_str()))
        .collect();
    let mut errors = Vec::new();
    for edge in &graph.edges {
        let name = format!("edge `{}` -{}-> `{}`", edge.from, edge.kind, edge.to);
        let (from_kinds, to_kinds): (&[&str], &[&str]) = match edge.kind.as_str() {
            "implements" => (&["module"], &["context"]),
            "governs" => (&["adr"], &["module", "context", "test", "doc"]),
            "depends_on" => (&["module"], &["module"]),
            "tested_by" => (&["module"], &["test"]),
            "relates_to" => (&["context"], &["context"]),
            _ => {
                errors.push(format!(
                    "{name}: unknown kind `{}` (expected one of {EDGE_KINDS:?})",
                    edge.kind
                ));
                continue;
            }
        };
        for (end, id, allowed) in [("from", &edge.from, from_kinds), ("to", &edge.to, to_kinds)] {
            match kinds.get(id.as_str()) {
                None => errors.push(format!("{name}: unknown node id `{id}`")),
                Some(kind) if !allowed.contains(kind) => errors.push(format!(
                    "{name}: `{end}` is a `{kind}`, expected one of {allowed:?}"
                )),
                Some(_) => {}
            }
        }
    }
    assert_no_errors(errors);
}

#[test]
fn no_duplicate_edges() {
    let graph = load();
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for edge in &graph.edges {
        if !seen.insert((&edge.from, &edge.to, &edge.kind)) {
            errors.push(format!(
                "duplicate edge `{}` -{}-> `{}`",
                edge.from, edge.kind, edge.to
            ));
        }
    }
    assert_no_errors(errors);
}

#[test]
fn every_module_implements_exactly_one_context() {
    let graph = load();
    let mut errors = Vec::new();
    for node in graph.nodes.iter().filter(|node| node.kind == "module") {
        let count = graph
            .edges
            .iter()
            .filter(|edge| edge.kind == "implements" && edge.from == node.id)
            .count();
        if count != 1 {
            errors.push(format!(
                "module `{}` has {count} `implements` edges, expected 1",
                node.id
            ));
        }
    }
    assert_no_errors(errors);
}

#[test]
fn every_adr_governs_a_node() {
    let graph = load();
    let mut errors = Vec::new();
    for node in graph.nodes.iter().filter(|node| node.kind == "adr") {
        let governs = graph
            .edges
            .iter()
            .any(|edge| edge.kind == "governs" && edge.from == node.id);
        if !governs {
            errors.push(format!("ADR `{}` has no `governs` edge", node.id));
        }
    }
    assert_no_errors(errors);
}
