//! Symlink whose target leaves the repo is PATH_UNCANONICAL.

#![cfg(unix)]

use leash::paths::{canonicalize_in_repo, PathError};
use std::fs;

#[test]
fn symlink_out_of_repo_is_denied() {
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("secret.txt"), "s").unwrap();
    std::os::unix::fs::symlink(outside.path().join("secret.txt"), tmp.path().join("link.txt"))
        .unwrap();
    assert_eq!(
        canonicalize_in_repo(tmp.path(), "link.txt").unwrap_err(),
        PathError::EscapesRepo
    );
}
