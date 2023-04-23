mod cli;
mod types;
mod utils;
mod ninjagen;

use clap::Parser;
use cli::Opts;

fn main() {
  let opts: Opts = Opts::parse();
  let mut ng = ninjagen::NinjaGen::new(opts.into()).unwrap();
  ng.generate().unwrap();
}
