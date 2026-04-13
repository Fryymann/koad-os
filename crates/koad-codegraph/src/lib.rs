//! # KoadOS Code Knowledge Graph (DEPRECATED)
//!
//! **NOTICE:** This crate is deprecated in favor of the `code-review-graph` 
//! Dynamic System Map (DSM) integration. 
//!
//! Provides AST-based symbol indexing and querying using tree-sitter.
//! This tool enables agents to perform high-fidelity codebase navigation
//! (e.g., "Find Definition", "List Traits") without the token cost of
//! reading raw source files.
//!
//! ## Architecture
//! - **CodeParser**: Uses `tree-sitter-rust` to extract functions, structs, and traits.
//! - **SQLite Index**: Stores symbol locations and metadata for rapid gRPC retrieval.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use tree_sitter::{Parser, Query, QueryCursor};

/// Represents a code symbol (function, struct, trait, etc.)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// The CodeGraph manages a persistent index of project symbols.
pub struct CodeGraph {
    db: Arc<Mutex<Connection>>,
}

impl CodeGraph {
    /// Create or open a code graph at the specified path.
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Self::init_db(conn)
    }

    /// Create an in-memory code graph for testing.
    pub fn new_with_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init_db(conn)
    }

    fn init_db(conn: Connection) -> Result<Self> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                path TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols (name)",
            [],
        )?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
        })
    }

    /// Index all Rust files in a project directory.
    pub fn index_project(&self, root: &Path) -> Result<()> {
        info!(path = ?root, "Indexing project code graph");
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|s| s == "rs").unwrap_or(false))
        {
            let path = entry.path();
            if let Ok(content) = std::fs::read_to_string(path) {
                self.index_file(path, &content)?;
            }
        }
        Ok(())
    }

    /// Index a single Rust file.
    pub fn index_file(&self, path: &Path, content: &str) -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::language())
            .context("Failed to load Rust grammar")?;

        let tree = parser
            .parse(content, None)
            .context("Failed to parse file")?;

        // Query for functions and structs
        let query_str = "(function_item name: (identifier) @name) @func
                         (struct_item name: (type_identifier) @name) @struct
                         (trait_item name: (type_identifier) @name) @trait";

        let query = Query::new(&tree_sitter_rust::language(), query_str)?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        let conn = self
            .db
            .lock()
            .map_err(|_| anyhow::anyhow!("DB lock poisoned"))?;
        let rel_path = path.to_string_lossy().to_string();

        for m in matches {
            // captures: 0=@func/struct/trait, 1=@name
            let node = m.captures[0].node;
            let name_node = m.captures[1].node;

            let name = &content[name_node.byte_range()];
            let kind = query.capture_names()[m.captures[0].index as usize].to_string();
            let range = node.range();

            conn.execute(
                "INSERT OR REPLACE INTO symbols (name, kind, path, start_line, end_line) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    name,
                    kind,
                    rel_path,
                    range.start_point.row + 1,
                    range.end_point.row + 1
                ],
            )?;
        }

        debug!(path = %rel_path, "Indexed file");
        Ok(())
    }

    /// Query the graph for a symbol by name.
    pub fn query_symbol(&self, name: &str) -> Result<Vec<Symbol>> {
        let conn = self
            .db
            .lock()
            .map_err(|_| anyhow::anyhow!("DB lock poisoned"))?;
        let mut stmt = conn.prepare(
            "SELECT name, kind, path, start_line, end_line FROM symbols WHERE name = ?1",
        )?;

        let symbols = stmt
            .query_map(params![name], |row| {
                Ok(Symbol {
                    name: row.get(0)?,
                    kind: row.get(1)?,
                    path: row.get(2)?,
                    start_line: row.get(3)?,
                    end_line: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(symbols)
    }

    /// Provides a summary of all symbols in a specific crate (directory).
    pub fn get_crate_summary(&self, crate_path: &str) -> Result<String> {
        let conn = self.db.lock().map_err(|_| anyhow::anyhow!("DB lock poisoned"))?;
        let mut stmt = conn.prepare(
            "SELECT path, name, kind FROM symbols WHERE path LIKE ?1 ORDER BY path, kind",
        )?;

        let pattern = format!("{}%", crate_path);
        let mut current_path = String::new();
        let mut summary = String::new();

        let rows = stmt.query_map(params![pattern], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        for row in rows {
            let (path, name, kind) = row?;
            if path != current_path {
                current_path = path.clone();
                summary.push_str(&format!("\nFile: {}\n", current_path));
            }
            summary.push_str(&format!("  - {}: {}\n", kind, name));
        }

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegraph_indexes_function() -> Result<()> {
        let graph = CodeGraph::new_with_memory()?;
        let content = "fn my_cool_func() { println!(\"hello\"); }";
        let path = Path::new("test.rs");

        graph.index_file(path, content)?;

        let symbols = graph.query_symbol("my_cool_func")?;
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "my_cool_func");
        assert_eq!(symbols[0].kind, "func");
        assert_eq!(symbols[0].start_line, 1);

        // Test Crate Summary
        let summary = graph.get_crate_summary("test")?;
        assert!(summary.contains("File: test.rs"));
        assert!(summary.contains("func: my_cool_func"));

        Ok(())
    }
}
