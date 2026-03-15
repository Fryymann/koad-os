//! Sanctuary Logic
//!
//! Enforces security boundaries, including path traversal protection ("Sanctuary Rule")
//! and access control for protected state keys.

use std::path::{Component, Path, PathBuf};
use tonic::Status;

/// Protected Redis/state keys that only Admin (tier 1) can write to.
const PROTECTED_KEYS: &[&str] = &[
    "identities",
    "identity_roles",
    "knowledge",
    "principles",
    "canon_rules",
];

/// Check if a state key is protected and whether the caller has sufficient tier.
pub fn check_protected_key(key: &str, caller_tier: Option<i32>) -> Result<(), Status> {
    if PROTECTED_KEYS.iter().any(|k| key.contains(k)) {
        let tier = caller_tier.unwrap_or(3); 
        if tier > 1 {
            return Err(Status::permission_denied(format!(
                "Key '''{}''' is protected: Admin (tier 1) only. Caller tier: {}",
                key, tier
            )));
        }
    }
    Ok(())
}

/// Normalizes a path, resolving "." and ".." components lexically.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(Component::RootDir) = components.peek() {
        components.next();
        PathBuf::from("/")
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Normal(c) => {
                ret.push(c);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            _ => {}
        }
    }
    ret
}

/// Validate that a file path is securely contained within the agent'''s workspace root.
pub fn validate_path(path: &str, workspace_root: &str) -> Result<(), Status> {
    let root = Path::new(workspace_root);
    let target = normalize_path(Path::new(path));

    if target.is_absolute() && !target.starts_with(root) {
        return Err(Status::permission_denied(format!(
            "Path '''{}''' is outside KOAD_WORKSPACE_ROOT '''{}'''",
            path, workspace_root
        )));
    }

    Ok(())
}
