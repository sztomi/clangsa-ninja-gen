use anyhow::Result;
use clap::{arg, Command};
use nom::{
  bytes::complete::{tag, take, take_until},
  character::complete::space1,
  combinator::{map_res, rest},
  IResult,
};
use std::{
  collections::HashSet,
  fmt::Display,
  fs::{File, OpenOptions},
  hash::Hash,
  io::{BufRead, BufReader, BufWriter, Write},
  path::PathBuf,
  str::FromStr,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ExtDefMapping {
  pub length: usize,
  pub usr: String,
  pub path: String,
}

impl Hash for ExtDefMapping {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.usr.hash(state);
  }
}

impl Display for ExtDefMapping {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{} {}", self.length, self.usr, self.path)
  }
}

fn parse_length(input: &str) -> IResult<&str, usize> {
  map_res(take_until(":"), usize::from_str)(input)
}

fn parse_key(length: usize, input: &str) -> IResult<&str, String> {
  let (input, key) = take(length)(input)?;
  Ok((input, key.to_string()))
}

pub fn parse_line(input: &str) -> IResult<&str, ExtDefMapping> {
  let (input, length) = parse_length(input)?;
  let (input, _) = tag(":")(input)?;
  let (input, key) = parse_key(length, input)?;
  let (input, _) = space1(input)?;
  let (input, path) = rest(input)?;

  Ok((
    input,
    ExtDefMapping {
      length,
      usr: key,
      path: path.to_string(),
    },
  ))
}

fn parse_response_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<PathBuf>> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);

  let mut paths = Vec::new();
  for line in reader.lines() {
    let line = line?;
    for path_str in line.split_whitespace() {
      let path = PathBuf::from(path_str);
      paths.push(path);
    }
  }

  Ok(paths)
}

fn main() -> Result<()> {
  let matches = Command::new("myprogram")
    .about("A program that accepts input files and an output file")
    .arg(
      arg!(
          [inputs]... "Input files or a response file (prefixed with @)"
      )
      .required(true),
    )
    .arg(
      arg!(
          [output_file] "Output file"
      )
      .required(true),
    )
    .get_matches();

  let inputs: Vec<String> = matches
    .get_many::<String>("inputs")
    .unwrap()
    .map(|s| s.to_string())
    .collect();

  let output_file = PathBuf::from(matches.get_one::<String>("output_file").unwrap());

  let input_files: Vec<PathBuf> = inputs
    .into_iter()
    .flat_map(|input| {
      if let Some(response_file) = input.strip_prefix('@') {
        parse_response_file(response_file).unwrap().into_iter()
      } else {
        vec![PathBuf::from(input)].into_iter()
      }
    })
    .collect();

  let defs = input_files
    .into_iter()
    .flat_map(|file| {
      let file = BufReader::new(File::open(file).unwrap());
      file.lines()
        .map(|line| parse_line(&line.unwrap()).unwrap().1)
        .collect::<HashSet<ExtDefMapping>>()
    })
    .collect::<HashSet<ExtDefMapping>>();

  let mut writer = BufWriter::new(
    OpenOptions::new()
      .write(true)
      .create(true)
      .open(output_file)?,
  );

  for d in defs {
    writeln!(writer, "{}", d)?;
  }
  Ok(())
}
