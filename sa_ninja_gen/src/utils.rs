use std::env;

use anyhow::{Context, Result};
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

pub fn find_command(command: &str, varname: &str) -> Result<PathBuf> {
  if let Ok(epath) = env::var(varname) {
    let path = Path::new(&epath);
    if path.exists() {
      return Ok(path.to_path_buf());
    } else {
      let path = which::which(epath).with_context(|| {
        format!(
          "Could not find '{}' in PATH ({} was set to this value)",
          command, varname
        )
      })?;
      return Ok(path);
    }
  }

  let path = which::which(command).with_context(|| {
    format!(
      "Could not find '{}' in PATH (and {} was not set)",
      command, varname
    )
  })?;

  Ok(path)
}

#[test]
fn test_vector_hash() {
  let data = vec!["a".to_string(), "b".to_string(), "c".to_string()];
  assert_eq!(vector_hash(&data), "78af5f94892f3950");
}

pub fn get_output_filename(
  output_dir: &Path,
  input_file: &str,
  prefix: &Path,
  extension: &str,
) -> Result<String> {
  let filename = output_dir
    .join(
      PathBuf::from(input_file)
        .strip_prefix(prefix)
        .with_context(|| {
          format!(
            "Failed to strip prefix {} from {}",
            prefix.display(),
            input_file
          )
        })?,
    )
    .with_extension(extension)
    .absolutize()
    .to_string_lossy()
    .to_string();
  Ok(filename)
}

#[test]
fn test_get_output_filename() {
  let output_dir = Path::new("/tmp");
  let input_file = "/tmp/foo/bar/baz.cpp";
  let prefix = Path::new("/tmp/foo");
  let extension = "ast";
  let filename = get_output_filename(output_dir, input_file, prefix, extension).unwrap();
  assert_eq!(filename, "/tmp/bar/baz.ast");

  let prefix = Path::new("/home");
  let filename = get_output_filename(output_dir, input_file, prefix, extension);
  assert!(filename.is_err());
}
