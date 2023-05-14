use std::{fs::File, io::BufReader, path::Path};

use crate::utils::vector_hash;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawCompileCommand {
  pub directory: String,
  pub command: Option<String>,
  pub arguments: Option<Vec<String>>,
  pub file: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompileCommand {
  pub directory: String,
  pub file: String,
  pub flags: Vec<String>,
  pub compiler: String,
  pub output: String,
}

impl CompileCommand {
  pub fn from_raw(cmd: RawCompileCommand) -> Result<Self> {
    let directory = cmd.directory.clone();

    let mut flags = match cmd.arguments {
      Some(args) => args,
      None => {
        let flags = cmd
          .command
          .as_ref()
          .ok_or_else(|| anyhow!("No command or arguments field in command {:?}", &cmd))?
          .split(' ')
          .map(|s| s.to_string())
          .collect::<Vec<String>>();
        flags
      }
    };

    let compiler = flags[0].to_string();
    flags.remove(0);

    let mut output = String::new();
    if let Some(i) = flags.iter().position(|f| f == "-o") {
      output = flags[i + 1].to_string();
      flags.drain(i..i + 2);
    }
    if let Some(i) = flags.iter().position(|f| f == "-c") {
      flags.drain(i..i + 2);
    } else if let Some(i) = flags.iter().position(|f| *f == cmd.file) {
      flags.remove(i);
    }

    Ok(Self {
      directory,
      file: cmd.file,
      flags,
      compiler,
      output,
    })
  }

  pub fn hash(&self) -> String {
    vector_hash(&self.flags)
  }
}

pub fn load_cmdb(path: &Path) -> Result<Vec<CompileCommand>> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  let cmdb: Vec<RawCompileCommand> = serde_json::from_reader(reader).with_context(|| {
    format!(
      "Reading compilation database file {}",
      path.to_string_lossy()
    )
  })?;
  let mut cmds = Vec::new();
  for cmd in cmdb.into_iter() {
    cmds.push(CompileCommand::from_raw(cmd)?);
  }
  Ok(cmds)
}

#[test]
fn test_init() {
  let cmd = RawCompileCommand {
    directory: "/home/username/project".to_string(),
    command: Some(
      "clang -o /home/username/project/main.o -fPIC -c /home/username/project/main.c".to_string(),
    ),
    file: "/home/username/project/main.c".to_string(),
    ..Default::default()
  };
  let cmd = CompileCommand::from_raw(cmd).unwrap();
  assert_eq!(cmd.compiler, "clang");
  assert_eq!(cmd.output, "/home/username/project/main.o");
  assert_eq!(cmd.flags, vec!["-fPIC"]);
}

#[test]
fn test_init_no_minus_c() {
  let cmd = RawCompileCommand {
    directory: "/home/username/project".to_string(),
    command: Some(
      "clang -o /home/username/project/main.o -fPIC /home/username/project/main.c".to_string(),
    ),
    file: "/home/username/project/main.c".to_string(),
    ..Default::default()
  };
  let cmd = CompileCommand::from_raw(cmd).unwrap();
  assert_eq!(cmd.compiler, "clang");
  assert_eq!(cmd.output, "/home/username/project/main.o");
  assert_eq!(cmd.flags, vec!["-fPIC"]);
}
