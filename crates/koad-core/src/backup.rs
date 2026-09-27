//! Consistent SQLite backups for the Citadel data directory.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Back up every `*.db` file in `db_dir` into `dest_root/<stamp>/`.
///
/// Uses `VACUUM INTO`, which reads a consistent snapshot through SQLite
/// (including pages still in the WAL), so it is safe while services are
/// writing. A plain file copy of a WAL-mode database can miss recent writes.
///
/// Returns the paths of the backup files written.
pub fn backup_databases(db_dir: &Path, dest_root: &Path, stamp: &str) -> Result<Vec<PathBuf>> {
    let dest = dest_root.join(stamp);
    if dest.exists() {
        anyhow::bail!("Backup destination already exists: {}", dest.display());
    }
    std::fs::create_dir_all(&dest)
        .with_context(|| format!("Failed to create {}", dest.display()))?;

    let mut sources: Vec<PathBuf> = std::fs::read_dir(db_dir)
        .with_context(|| format!("Failed to read {}", db_dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "db"))
        .collect();
    sources.sort();

    let mut written = Vec::with_capacity(sources.len());
    for src in sources {
        let target = dest.join(src.file_name().expect("db file has a name"));
        let conn = rusqlite::Connection::open(&src)
            .with_context(|| format!("Failed to open {}", src.display()))?;
        conn.execute("VACUUM INTO ?1", [target.to_string_lossy().as_ref()])
            .with_context(|| format!("Failed to back up {}", src.display()))?;
        written.push(target);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn backup_captures_uncheckpointed_wal_writes_and_skips_non_db_files() {
        let src = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();

        // A WAL-mode database with a committed row that has not been
        // checkpointed into the main file. Keep the connection open so
        // SQLite does not checkpoint on close.
        let live = Connection::open(src.path().join("citadel.db")).unwrap();
        live.pragma_update(None, "journal_mode", "WAL").unwrap();
        live.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
        live.execute_batch("CREATE TABLE t(v TEXT); INSERT INTO t VALUES ('recent');")
            .unwrap();

        std::fs::write(src.path().join("notes.txt"), "not a database").unwrap();
        std::fs::write(src.path().join("cass.db.pre-migration"), "old copy").unwrap();

        let files = backup_databases(src.path(), dest.path(), "20260926-120000").unwrap();

        let expected = dest.path().join("20260926-120000").join("citadel.db");
        assert_eq!(files, vec![expected.clone()]);

        let restored = Connection::open(&expected).unwrap();
        let v: String = restored
            .query_row("SELECT v FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, "recent");
        drop(live);
    }

    #[test]
    fn backup_refuses_to_overwrite_an_existing_snapshot() {
        let src = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        Connection::open(src.path().join("koad.db"))
            .unwrap()
            .execute_batch("CREATE TABLE t(v);")
            .unwrap();

        backup_databases(src.path(), dest.path(), "same").unwrap();
        assert!(backup_databases(src.path(), dest.path(), "same").is_err());
    }
}
