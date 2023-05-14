use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use ninja_syntax::{Build, Rule, Variable, Writer};
use sugar_path::SugarPath;

use crate::cli::OptsClean;
use crate::cmdb::{load_cmdb, CompileCommand};
use crate::utils::{find_command, get_output_filename, vector_hash};

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
    let cmdb: Vec<CompileCommand> = load_cmdb(&opts.compile_commands)?;

    let cem_command = find_command("clang-extdef-mapping", "CLANG_EXTDEF_MAPPING")?;
    let merge_command = find_command("merge_extdefs", "MERGE_EXTDEFS")?;

    let variables = vec![
      Variable::new("root", &opts.repo.to_string_lossy(), 0),
      Variable::new("cem", &cem_command.to_string_lossy(), 0),
      Variable::new("merge_extdefs", &merge_command.to_string_lossy(), 0),
    ];
    let rules = HashMap::from([
      (
        "cem".to_string(),
        Rule::new("cem", "$cem $in > $out").description("CEM $in"),
      ),
      (
        "merge".to_string(),
        Rule::new("merge", "$merge_extdefs @extdefs.rsp $out")
          .description("MERGE $in")
          .rspfile("extdefs.rsp")
          .rspfile_content("$in"),
      ),
    ]);
    let mut commands = HashMap::new();
    let mut pch_commands = HashMap::new();
    for cmd in cmdb.into_iter() {
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

  fn analyze_rule(
    rules: &mut HashMap<String, Rule>,
    cmd: &CompileCommand,
    outdir: &Path,
    ctu: bool,
  ) -> String {
    let mut flags = cmd.flags.clone();
    // TODO(sztomi): read the analyzer options from a config file
    flags.extend(
      [
        "--analyze",
        "-Xclang",
        "-analyzer-config",
        "-Xclang",
        "expand-macros=true",
        "-Xclang",
        "-analyzer-config",
        "-Xclang",
        "aggressive-binary-operation-simplification=true",
        "-Xclang",
        "-analyzer-output=plist-multi-file",
      ]
      .map(|x| x.to_string()),
    );

    if ctu {
      flags.extend(
        [
          "-Xclang",
          "-analyzer-config",
          "-Xclang",
          "experimental-enable-naive-ctu-analysis=true",
          "-Xclang",
          "-analyzer-config",
          "-Xclang",
          &format!("ctu-dir={}", outdir.to_string_lossy()),
          // "-Xclang", "-analyzer-config",
          // "-Xclang", "crosscheck-with-z3=true",
          // "-Xclang", "-analyzer-opt-analyze-headers",
        ]
        .map(|x| x.to_string()),
      )
    }

    flags.extend(["-o", "$out", "$in"].map(|x| x.to_string()));

    let hash = vector_hash(&flags);
    let name = format!("analyze_{}", hash);
    if !rules.contains_key(&hash) {
      rules.insert(
        name.clone(),
        Rule::new(&name, &format!("{} {}", cmd.compiler, flags.join(" ")))
          .description("ANALYZE $in")
          .pool("analyze"),
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
      self
        .builds
        .push(Build::new(&[&output_filename], &rule).inputs(&[&cmd.file]));
      pchs.push(cmd.output.clone());
    }

    let all_extdefs_file = self
      .opts
      .output_dir
      .join("externalDefMap.txt")
      .absolutize()
      .to_string_lossy()
      .to_string();

    for (file, command) in self.commands.iter() {
      let ast_rule = Self::ast_rule(&mut self.rules, command);

      // The output filename will be the same as the input filename, but relative to the output dir,
      // and with the extension changed to .ast. i.e. we are replicating the source tree under the
      // output directory, in the ASTs subdirectory.
      let ast_filename = get_output_filename(
        &self.opts.output_dir.join("ASTs"),
        file,
        &self.opts.repo,
        "ast",
      )?;

      // We collect the actual PCH deps for the given file, and add them as implicit dependencies.
      let pch_deps: Vec<&str> = command
        .flags
        .iter()
        .enumerate()
        .filter(|(_, x)| x == &"-include-pch")
        .map(|(i, _)| command.flags[i + 2].as_str())
        .collect();
      let build = Build::new(&[&ast_filename], &ast_rule)
        .inputs(&[&command.file])
        .implicit(&pch_deps);

      asts.push(ast_filename.clone());
      self.builds.push(build);

      // now we generate the extdef files for each ast file
      let extdef = get_output_filename(
        &self.opts.output_dir.join("extdefs"),
        file,
        &self.opts.repo,
        "extdef",
      )?;

      self
        .builds
        .push(Build::new(&[&extdef], "cem").inputs(&[&ast_filename]));
      extdefs.push(extdef);

      let analyze_rule = Self::analyze_rule(
        &mut self.rules,
        command,
        &self.opts.output_dir,
        self.opts.ctu,
      );

      let analyze_result = get_output_filename(
        &self.opts.output_dir.join("reports"),
        file,
        &self.opts.repo,
        "plist",
      )?;

      self.builds.push(
        Build::new(&[&analyze_result], &analyze_rule)
          .inputs(&[&command.file])
          .implicit(&[&all_extdefs_file]),
      );
    }

    ninja.pool("analyze", self.opts.ctu_pool);
    ninja.write_variables(&self.variables, false);
    ninja.newline();
    for rule in self.rules.values() {
      ninja.rule(rule);
      ninja.newline();
    }
    ninja.write_builds(&self.builds, true);

    // merge all extdef files into a single file
    ninja.build(
      &Build::new(&[&all_extdefs_file], "merge")
        .inputs(&extdefs.iter().map(|s| s.as_str()).collect::<Vec<&str>>()),
    );

    Ok(())
  }
}
