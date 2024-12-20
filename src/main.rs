use std::{
  collections::{HashMap, HashSet},
  io::Read,
  iter::Peekable,
  str::Chars,
};

use clap::Parser;

pub struct Matcher<'input> {
  input: &'input str,
  peekable: Peekable<Chars<'input>>,
  index: usize,
  start: usize,
}

#[inline(always)]
fn word(c: &char) -> bool {
  c.is_alphabetic()
}

impl<'input> Matcher<'input> {
  fn new(input: &'input str) -> Self {
    Self {
      input,
      peekable: input.chars().peekable(),
      index: 0,
      start: 0,
    }
  }

  fn save(&mut self) {
    self.start = self.index;
  }

  fn peek(&mut self) -> Option<&char> {
    self.peekable.peek()
  }

  fn advance(&mut self) -> Option<char> {
    let char = self.peekable.next()?;
    self.index += char.len_utf8();
    Some(char)
  }

  fn skip(&mut self) {
    while let Some(char) = self.peek() {
      if !word(char) {
        self.advance();
      } else {
        break;
      }
    }
  }

  fn word(&mut self) -> &'input str {
    while let Some(char) = self.peek() {
      if word(char) {
        self.advance();
      } else {
        break;
      }
    }
    &self.input[self.start..self.index]
  }

  fn next_word(&mut self) -> Option<&'input str> {
    self.skip();
    self.save();

    if self.peek().is_some() {
      Some(self.word())
    } else {
      None
    }
  }
}

#[derive(Debug, Default, serde::Serialize)]
#[serde(transparent)]
struct Indexes<'a> {
  indexes: HashMap<String, HashSet<&'a std::path::Path>>,
}

impl<'a> Indexes<'a> {
  fn new() -> Self {
    Self::default()
  }

  fn add_file(&mut self, path: &'a std::path::Path) -> std::io::Result<()> {
    let mut buf = String::with_capacity(8192);
    let mut file = std::fs::File::open(path)?;
    file.read_to_string(&mut buf)?;

    let mut matcher = Matcher::new(&buf);
    while let Some(word) = matcher.next_word() {
      self
        .indexes
        .entry(word.to_owned())
        .or_default()
        .insert(path);
    }

    Ok(())
  }
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  glob_pat: String,
  #[arg(short, long)]
  output: Option<String>,
}

const OUTPUT_FILE: &'static str = "./rino.output.json";

fn main() {
  let cli = Cli::parse();

  if let Err(e) = run(cli.glob_pat, cli.output) {
    eprintln!("{e}");
  }
}

fn run<'a>(glob_pat: String, output: Option<String>) -> std::io::Result<()> {
  let mut indexes = Indexes::new();

  let glob: Vec<_> = glob::glob(&glob_pat)
    .map_err(|e| std::io::Error::other(e))?
    .collect();

  for entry in &glob {
    match entry {
      Ok(path) => indexes.add_file(path.as_path())?,
      Err(e) => eprintln!("Error: {e}"),
    }
  }

  let output = std::fs::File::options()
    .create(true)
    .write(true)
    .open(output.unwrap_or(String::from(OUTPUT_FILE)))?;
  serde_json::to_writer_pretty(output, &indexes)?;

  Ok(())
}
