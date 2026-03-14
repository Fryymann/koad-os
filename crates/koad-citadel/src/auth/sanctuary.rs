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
///
/// # Errors
/// Returns `PERMISSION_DENIED` if the caller's tier is insufficient for the key.
pub fn check_protected_key(key: &str, caller_tier: Option<i32>) -> Result<(), Status> {
    if PROTECTED_KEYS.iter().any(|k| key.contains(k)) {
        let tier = caller_tier.unwrap_or(3); // Default to lowest privilege
        if tier > 1 {
            return Err(Status::permission_denied(format!(
                "Key '{}' is protected: Admin (tier 1) only. Caller tier: {}",
                key, tier
            )));
        }
    }
    Ok(())
}

/// Normalizes a path, resolving `.` and `..` components lexically.
/// This prevents path traversal attacks like `/path/to/../../etc/passwd`.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::RootDir) = components.peek().clone() {
        components.next();
        PathBuf::from(c.as_os_str())
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
            Component::RootDir => {
                // This case should be handled by the initial peek
            }
            Component::Prefix(..) => {
                // Not supported on Unix
            }
        }
    }
    ret
}

/// Validate that a file path is securely contained within the agent's workspace root.
///
/// # Errors
/// Returns `PERMISSION_DENIED` if the path escapes the sandbox.
pub fn validate_path(path: &str, workspace_root: &str) -> Result<(), Status> {
    let root = Path::new(workspace_root);
    let target = normalize_path(Path::new(path));

    // If the path is absolute, it must be a sub-path of the root.
    // If relative, it's resolved from the root, so containment is implicit.
    if target.is_absolute() && !target.starts_with(root) {
        return Err(Status::permission_denied(format!(
            "Path '{}' is outside KOAD_WORKSPACE_ROOT '{}'",
            path, workspace_root
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_protected_key_admin_allowed() {
        assert!(check_protected_key("koad:identities:tyr", Some(1)).is_ok());
    }

    #[test]
    fn test_protected_key_agent_denied() {
        let err = check_protected_key("koad:identities:tyr", Some(2)).unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    #[test]
    fn test_unprotected_key_allowed() {
        assert!(check_protected_key("koad:session:abc", Some(3)).is_ok());
    }

    #[test]
    fn path_validation_allows_safe_paths() {
        assert!(validate_path("safe/file.txt", "/app").is_ok());
        assert!(validate_path("/app/foo/bar.txt", "/app").is_ok());
    }

    #[test]
    fn path_validation_denies_traversal() {
        let res = validate_path("../../../etc/passwd", "/app/safe/dir");
        assert_eq!(res.unwrap_err().code(), tonic::Code::PermissionDenied);

        let res2 = validate_path("/etc/passwd", "/app");
        assert_eq!(res2.unwrap_err().code(), tonic::Code::PermissionDenied);
    }
}
