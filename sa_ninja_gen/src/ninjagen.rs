use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::Result;
use ninja_syntax::{Build, Rule, Variable, Writer};
use sugar_path::SugarPath;
use which::which;

use crate::cli::OptsClean;
use crate::types::CompileCommand;
use crate::utils::vector_hash;

pub struct NinjaGen {
  opts: OptsClean,
  rules: HashMap<String, Rule>,
  builds: Vec<Build>,
  variables: Vec<Variable>,
  commands: HashMap<String, CompileCommand>,
  pch_commands: HashMap<String, CompileCommand>,
}

impl NinjaGen {
  pub fn new(opts: OptsClean) -> Result<Self> {
    let file = File::open(&opts.compile_commands)?;
    let reader = BufReader::new(file);
    let cmdb: Vec<CompileCommand> = serde_json::from_reader(reader)?;

    let variables = vec![
      Variable::new("root", &opts.repo.to_string_lossy(), 0),
      Variable::new("cem", &which("clang-extdef-mapping")?.to_string_lossy(), 0),
    ];
    let rules = HashMap::from([
      (
        "cem".to_string(),
        Rule::new("cem", "$cem $in > $out").description("CEM $in"),
      ),
      (
        "collect".to_string(),
        Rule::new("collect", "cat $in > $out").description("COLLECT $in"),
      ),
      (
        "merge".to_string(),
        Rule::new("merge", "merge_extdefs $in $out")
          .description("MERGE $in")
          .rspfile("extdefs.rsp")
          .rspfile_content("$in"),
      ),
    ]);
    let mut commands = HashMap::new();
    let mut pch_commands = HashMap::new();
    for mut cmd in cmdb.into_iter() {
      cmd.init();

      if opts.detect_pch && cmd.flags.contains(&"-emit-pch".to_string()) {
        pch_commands.insert(cmd.file.clone(), cmd.clone());
      }

      commands.insert(cmd.file.clone(), cmd);
    }

    Ok(Self {
      opts,
      rules,
      builds: Vec::new(),
      variables,
      commands,
      pch_commands,
    })
  }

  fn ast_rule(rules: &mut HashMap<String, Rule>, cmd: &CompileCommand) -> String {
    let mut flags = cmd.flags.clone();
    flags.extend(["-emit-ast", "-c", "$in", "-o", "$out"].map(|x| x.to_string()));
    let hash = vector_hash(&flags);
    let name = format!("ast_{}", hash);
    if !rules.contains_key(&hash) {
      rules.insert(
        name.clone(),
        Rule::new(&name, &format!("{} {}", cmd.compiler, flags.join(" "))).description("AST $in"),
      );
    }
    name
  }

  fn pch_rule(rules: &mut HashMap<String, Rule>, cmd: &CompileCommand) -> String {
    let hash = cmd.hash();
    let name = format!("pch_{}", hash);
    if !rules.contains_key(&hash) {
      rules.insert(
        name.clone(),
        Rule::new(
          &name,
          &format!("{} {} -o $out -c $in", cmd.compiler, cmd.flags.join(" ")),
        )
        .description("PCH $in"),
      );
    }
    name
  }

  pub fn generate(&mut self) -> Result<()> {
    let mut ninja = Writer::new(&self.opts.output_file);
    let mut pchs = Vec::new();
    let mut asts = Vec::new();
    let mut extdefs = Vec::new();

    for cmd in self.pch_commands.values() {
      let rule = Self::pch_rule(&mut self.rules, cmd);
      let output_filename = PathBuf::from(&cmd.output)
        .absolutize()
        .to_string_lossy()
        .to_string();
      println!("PCH: {}", &output_filename);
      self
        .builds
        .push(Build::new(&[&output_filename], &rule).inputs(&[&cmd.file]));
      pchs.push(cmd.output.clone());
    }

    for (file, command) in self.commands.iter() {
      let ast_rule = Self::ast_rule(&mut self.rules, command);

      // The output filename will be the same as the input filename, but relative to the output dir,
      // and with the extension changed to .ast. i.e. we are replicating the source tree under the
      // output directory, in the ASTs subdirectory.
      let output_filename = self
        .opts
        .output_dir
        .join("ASTs")
        .join(PathBuf::from(file).strip_prefix(&self.opts.repo).unwrap())
        .with_extension("ast")
        .absolutize()
        .to_string_lossy()
        .to_string();

      // We collect the actual PCH deps for the given file, and add them as implicit dependencies.
      let pch_deps: Vec<&str> = command
        .flags
        .iter()
        .enumerate()
        .filter(|(_, x)| x == &"-include-pch")
        .map(|(i, _)| command.flags[i + 2].as_str())
        .collect();
      let build = Build::new(&[&output_filename], &ast_rule)
        .inputs(&[&command.file])
        .implicit(&pch_deps);

      asts.push(output_filename.clone());
      self.builds.push(build);

      // now we generate the extdef files for each ast file
      let extdef = self
        .opts
        .output_dir
        .join("extdefs")
        .join(PathBuf::from(file).strip_prefix(&self.opts.repo).unwrap())
        .with_extension("extdef")
        .absolutize()
        .to_string_lossy()
        .to_string();

      self
        .builds
        .push(Build::new(&[&extdef], "cem").inputs(&[&output_filename]));
      extdefs.push(extdef);
    }

    ninja.write_variables(&self.variables, false);
    ninja.newline();
    for rule in self.rules.values() {
      ninja.rule(rule);
      ninja.newline();
    }
    ninja.write_builds(&self.builds, true);

    // merge all extdef files into a single file
    ninja.build(
      &Build::new(&["externalDefMap.txt"], "merge")
        .inputs(&extdefs.iter().map(|s| s.as_str()).collect::<Vec<&str>>()),
    );

    Ok(())
  }
}
