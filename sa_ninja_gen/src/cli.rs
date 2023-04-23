use std::path::PathBuf;
use clap::{Parser, ArgAction};
use sugar_path::SugarPath;

use crate::utils;

#[derive(Parser)]
#[command(
  version = "1.0",
  author = "Tamás Szelei",
  about = "Generates a ninja build to run clang static analyzer."
)]
pub struct Opts {
  /// Enable CTU
  #[arg(short, long)]
  pub ctu: bool,

  /// Turns off generating build commands for PCH files (-emit-pch).
  /// The analysis might be broken if your build uses PCH files and you turn this off.
  #[arg(long = "no-pch-detection", default_value_t = true, action = ArgAction::SetFalse)]
  pub detect_pch: bool,

  /// Path to the repository
  #[arg(short, long, value_name = "PATH")]
  pub repo: Option<PathBuf>,

  /// Path to compile_commands.json file
  pub compile_commands: PathBuf,

  /// Path to the output directory
  #[arg(short, long, value_name = "OUTPUT_DIR")]
  pub output_dir: Option<PathBuf>,

  /// Path to ctu.ninja file
  #[arg(value_name = "OUTPUT_FILE")]
  pub output_file: PathBuf,
}

pub struct OptsClean {
  pub ctu: bool,
  pub detect_pch: bool,
  pub repo: PathBuf,
  pub compile_commands: PathBuf,
  pub output_dir: PathBuf,
  pub output_file: PathBuf,
}

impl From<Opts> for OptsClean {
  fn from(opts: Opts) -> Self {
    let mut opts = opts;
    opts.init();
    Self {
      ctu: opts.ctu,
      detect_pch: opts.detect_pch,
      repo: opts.repo.unwrap(),
      compile_commands: utils::absolutize(&opts.compile_commands.normalize()),
      output_dir: opts.output_dir.unwrap_or_else(|| PathBuf::from(".")),
      output_file: opts.output_file,
    }
  }
}

impl Opts {
  pub fn init(&mut self) {
    self.output_file = utils::absolutize(&self.output_file);
    if self.output_dir.is_none() {
      self.output_dir = Some(self.output_file.parent().unwrap().to_path_buf());
    }
    if self.repo.is_none() {
      self.repo = Some(utils::find_repo_root().unwrap());
    }
  }
}
