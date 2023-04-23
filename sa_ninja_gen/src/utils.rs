use std::env;

use std::path::{Path, PathBuf};
use sugar_path::SugarPath;
use xxhash_rust::xxh3::Xxh3;

pub fn find_repo_root() -> Option<PathBuf> {
  let current_dir = env::current_dir().ok()?;
  let mut current: &Path = current_dir.as_path();

  while let Some(parent) = current.parent() {
    for vcs in &[".git", ".hg", ".svn", ".bzr"] {
      let marker = current.join(vcs);
      if marker.exists() {
        return Some(current.to_path_buf());
      }
    }
    current = parent;
  }

  None
}

pub fn absolutize(path: &Path) -> PathBuf {
  path.absolutize().into_owned()
}

pub fn vector_hash(data: &[String]) -> String {
  let hash: u64 = data
    .iter()
    .fold(Xxh3::default(), |mut hasher, string| {
      hasher.update(string.as_bytes());
      hasher
    })
    .digest();
  format!("{:x}", hash)
}

#[test]
fn test_vector_hash() {
  let data = vec!["a".to_string(), "b".to_string(), "c".to_string()];
  assert_eq!(vector_hash(&data), "78af5f94892f3950");
}