use serde::{Deserialize, Serialize};
use crate::utils::vector_hash;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileCommand {
  pub directory: String,
  pub command: String,
  pub file: String,
  #[serde(skip)]
  pub flags: Vec<String>,
  #[serde(skip)]
  pub compiler: String,
  #[serde(skip)]
  pub output: String,
}

impl CompileCommand {
  pub fn init(&mut self) {
    self.flags = self.command.split(' ').map(|s| s.to_string()).collect();
    self.compiler = self.flags[0].to_string();
    self.flags.remove(0);

    if let Some(i) = self.flags.iter().position(|f| f == "-o") {
      self.output = self.flags[i + 1].to_string();
      self.flags.drain(i..i + 2);
    }
    if let Some(i) = self.flags.iter().position(|f| f == "-c") {
      self.flags.drain(i..i + 2);
    }
    else if let Some(i) = self.flags.iter().position(|f| *f == self.file) {
      self.flags.remove(i);
    }
  }
  
  pub fn hash(&self) -> String {
    vector_hash(&self.flags)
  }
}

#[test]
fn test_init() {
  let mut cmd = CompileCommand {
    directory: "/home/username/project".to_string(),
    command: "clang -o /home/username/project/main.o -fPIC -c /home/username/project/main.c"
      .to_string(),
    file: "/home/username/project/main.c".to_string(),
    ..Default::default()
  };
  cmd.init();
  assert_eq!(cmd.compiler, "clang");
  assert_eq!(cmd.output, "/home/username/project/main.o");
  assert_eq!(cmd.flags, vec!["-fPIC"]);
}

#[test]
fn test_init_no_minus_c() {
  let mut cmd = CompileCommand {
    directory: "/home/username/project".to_string(),
    command: "clang -o /home/username/project/main.o -fPIC /home/username/project/main.c"
      .to_string(),
    file: "/home/username/project/main.c".to_string(),
    ..Default::default()
  };
  cmd.init();
  assert_eq!(cmd.compiler, "clang");
  assert_eq!(cmd.output, "/home/username/project/main.o");
  assert_eq!(cmd.flags, vec!["-fPIC"]);
}
