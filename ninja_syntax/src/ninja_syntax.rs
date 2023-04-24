use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use textwrap::{wrap, Options, WordSeparator, WordSplitter};

fn to_vec_string(in_vec: &[&str]) -> Vec<String> {
  in_vec.iter().map(|x| x.to_string()).collect()
}

fn split_at_spaces(word: &str) -> Vec<usize> {
  word.match_indices(' ').map(|(idx, _)| idx + 1).collect()
}

#[derive(Default, Clone)]
pub struct Rule {
  pub name: String,
  pub command: String,
  pub description: Option<String>,
  pub depfile: Option<String>,
  pub generator: bool,
  pub pool: Option<String>,
  pub restat: bool,
  pub rspfile: Option<String>,
  pub rspfile_content: Option<String>,
  pub deps: Option<String>,
}

impl Rule {
  pub fn new(name: &str, command: &str) -> Self {
    Rule {
      name: name.to_string(),
      command: command.to_string(),
      generator: false,
      restat: false,
      ..Default::default()
    }
  }

  pub fn name(mut self, val: &str) -> Self {
    self.name = val.to_string();
    self
  }

  pub fn command(mut self, val: &str) -> Self {
    self.command = val.to_string();
    self
  }

  pub fn description(mut self, val: &str) -> Self {
    self.description = Some(val.to_string());
    self
  }

  pub fn depfile(mut self, val: &str) -> Self {
    self.depfile = Some(val.to_string());
    self
  }

  pub fn generator(mut self, val: bool) -> Self {
    self.generator = val;
    self
  }

  pub fn pool(mut self, val: &str) -> Self {
    self.pool = Some(val.to_string());
    self
  }

  pub fn restat(mut self, val: bool) -> Self {
    self.restat = val;
    self
  }

  pub fn rspfile(mut self, val: &str) -> Self {
    self.rspfile = Some(val.to_string());
    self
  }

  pub fn rspfile_content(mut self, val: &str) -> Self {
    self.rspfile_content = Some(val.to_string());
    self
  }

  pub fn deps(mut self, val: &str) -> Self {
    self.deps = Some(val.to_string());
    self
  }
}

#[derive(Default)]
pub struct Build {
  outputs: Vec<String>,
  rule: String,
  inputs: Vec<String>,
  implicit: Vec<String>,
  order_only: Vec<String>,
  variables: HashMap<String, String>,
  implicit_outputs: Vec<String>,
  pool: Option<String>,
  dyndep: Option<String>,
}

impl Build {
  pub fn new(outputs: &[&str], rule: &str) -> Self {
    Build {
      outputs: to_vec_string(outputs),
      rule: rule.to_string(),
      ..Default::default()
    }
  }

  pub fn outputs(mut self, outputs: &[&str]) -> Self {
    self.outputs = to_vec_string(outputs);
    self
  }

  pub fn rule(mut self, rule: &str) -> Self {
    self.rule = rule.to_string();
    self
  }

  pub fn inputs(mut self, inputs: &[&str]) -> Self {
    self.inputs = to_vec_string(inputs);
    self
  }

  pub fn implicit(mut self, implicit: &[&str]) -> Self {
    self.implicit = to_vec_string(implicit);
    self
  }

  pub fn order_only(mut self, order_only: &[&str]) -> Self {
    self.order_only = to_vec_string(order_only);
    self
  }

  pub fn variables(mut self, variables: &HashMap<&str, &str>) -> Self {
    self.variables.clear();

    for (key, value) in variables {
      self.variables.insert(key.to_string(), value.to_string());
    }

    self
  }

  pub fn implicit_outputs(mut self, implicit_outputs: &[&str]) -> Self {
    self.implicit_outputs = to_vec_string(implicit_outputs);
    self
  }

  pub fn pool(mut self, pool: &str) -> Self {
    self.pool = Some(pool.to_string());
    self
  }

  pub fn dyndep(mut self, dyndep: &str) -> Self {
    self.dyndep = Some(dyndep.to_string());
    self
  }
}

pub struct Variable {
  pub name: String,
  pub value: String,
  pub indent: usize,
}

impl Variable {
  pub fn new(name: &str, value: &str, indent: usize) -> Self {
    Variable {
      name: name.to_string(),
      value: value.to_string(),
      indent,
    }
  }
}
pub struct Writer {
  #[allow(dead_code)]
  file_path: PathBuf,
  width: usize,
  memory_p: Vec<u8>,
}

impl Writer {
  pub fn new<P: AsRef<Path>>(file_path: &P) -> Self {
    Writer {
      file_path: file_path.as_ref().to_path_buf(),
      width: 78,
      memory_p: Vec::new(),
    }
  }

  fn write_line(&mut self, line: &str) {
    let out = format!("{}\n", line);
    self.memory_p.write_all(out.as_bytes()).unwrap();
  }

  pub fn as_str(&mut self) -> &str {
    std::str::from_utf8(&self.memory_p).unwrap()
  }

  pub fn close(&mut self) -> std::io::Result<()> {
    let mut fp = OpenOptions::new()
      .create(true)
      .write(true)
      .truncate(true)
      .open(&self.file_path)?;
    fp.write_all(&self.memory_p)?;
    Ok(())
  }

  fn wrapped_line(&mut self, text: &str, indent: usize) {
    let leading_space = "  ".repeat(indent);
    let subseq_space = "  ".repeat(indent + 1);
    let options = Options::new(self.width - indent * 2)
      .break_words(false)
      .initial_indent(&leading_space)
      .subsequent_indent(&subseq_space)
      .word_splitter(WordSplitter::Custom(split_at_spaces))
      .word_separator(WordSeparator::AsciiSpace);
    let out = wrap(text, options);

    // join the lines with the ninja line continuation character $
    let out = out.join(" $\n");
    let out = format!("{}\n", out);

    self.memory_p.write_all(out.as_bytes()).unwrap();
  }

  pub fn comment(&mut self, comment: &str) -> &mut Self {
    let sc = format!("# {}", comment);
    self.write_line(&sc);
    self
  }

  pub fn newline(&mut self) -> &mut Self {
    self.write_line("");
    self
  }

  pub fn variable(&mut self, key: &str, value: &str, indent: usize) -> &mut Self {
    let var = format!("{} = {}", key, value);
    self.wrapped_line(&var, indent);
    self
  }

  pub fn variable_list(&mut self, key: &str, value: &[&str], indent: usize) -> &mut Self {
    let value_str = value.join(" ");
    self.variable(key, &value_str, indent);
    self
  }

  pub fn pool(&mut self, name: &str, depth: u8) -> &mut Self {
    let out = format!("pool {}", name);
    self.write_line(&out);
    self.variable("depth", &format!("{}", depth), 1);
    self
  }

  pub fn rule(&mut self, rule: &Rule) -> &mut Self {
    let out = format!("rule {}", rule.name);
    self.wrapped_line(&out, 0);
    self.variable("command", &rule.command, 1);

    if let Some(desc) = &rule.description {
      if !desc.is_empty() {
        self.variable("description", desc, 1);
      }
    }

    if let Some(depfile) = &rule.depfile {
      if !depfile.is_empty() {
        self.variable("depfile", depfile, 1);
      }
    }

    if rule.generator {
      self.variable("generator", "1", 1);
    }

    if let Some(pool) = &rule.pool {
      if !pool.is_empty() {
        self.variable("pool", pool, 1);
      }
    }

    if rule.restat {
      self.variable("restat", "1", 1);
    }

    if let Some(rspfile) = &rule.rspfile {
      if !rspfile.is_empty() {
        self.variable("rspfile", rspfile, 1);
      }
    }

    if let Some(rspfile_content) = &rule.rspfile_content {
      if !rspfile_content.is_empty() {
        self.variable("rspfile_content", rspfile_content, 1);
      }
    }

    if let Some(deps) = &rule.deps {
      if !deps.is_empty() {
        self.variable("deps", deps, 1);
      }
    }

    self
  }

  fn escape_path(&mut self, word: &str) -> String {
    word
      .replace('$', "$$")
      .replace(' ', "$ ")
      .replace(':', "$:")
  }

  fn escape_strings(&mut self, vec: &[String]) -> Vec<String> {
    vec.iter().map(|x| self.escape_path(x)).collect()
  }

  pub fn build(&mut self, build: &Build) -> &mut Self {
    let mut outputs: Vec<String> = self.escape_strings(&build.outputs);
    let mut all_input: Vec<String> = self.escape_strings(&build.inputs);

    all_input.insert(0, build.rule.to_string());

    if !build.implicit.is_empty() {
      all_input.push("|".to_string());
      all_input.append(&mut self.escape_strings(&build.implicit));
    }

    if !build.order_only.is_empty() {
      all_input.push("||".to_string());
      all_input.append(&mut self.escape_strings(&build.order_only));
    }

    if !build.implicit_outputs.is_empty() {
      outputs.push("|".to_string());
      outputs.append(&mut self.escape_strings(&build.implicit_outputs));
    }

    let out = format!("build {}: {}", outputs.join(" "), all_input.join(" "));
    self.wrapped_line(&out, 0);

    if let Some(pool) = &build.pool {
      if !pool.is_empty() {
        self.variable("pool", pool, 1);
      }
    }

    if let Some(dyndep) = &build.dyndep {
      if !dyndep.is_empty() {
        self.variable("dyndep", dyndep, 1);
      }
    }

    if !build.variables.is_empty() {
      for (key, value) in &build.variables {
        self.variable(key, value, 1);
      }
    }

    self
  }

  // convenience function to write a collection of builds
  pub fn write_builds(&mut self, builds: &[Build], add_newlines: bool) -> &mut Self {
    for build in builds {
      self.build(build);
      if add_newlines {
        self.newline();
      }
    }
    self
  }

  // convenience function to write a collection of rules
  pub fn write_rules(&mut self, rules: &[Rule], add_newlines: bool) -> &mut Self {
    for rule in rules {
      self.rule(rule);
      if add_newlines {
        self.newline();
      }
    }
    self
  }

  // convenience function to write a collection of variables
  pub fn write_variables(&mut self, variables: &[Variable], add_newlines: bool) -> &mut Self {
    for variable in variables {
      self.variable(&variable.name, &variable.value, 0);
      if add_newlines {
        self.newline();
      }
    }
    self
  }
}

impl Drop for Writer {
  fn drop(&mut self) {
    self.close().unwrap()
  }
}
