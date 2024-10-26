mod inspect;
mod subset;

use std::fs;
use std::path::Path;

use clap::builder::PossibleValue;
use clap::{Parser, ValueEnum};

use crate::subset::Flavor;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(short, long, exclusive = true)]
  inspect: bool,

  /// List of subsets to include.
  #[arg(short, long, required = true, value_delimiter = ',')]
  subsets: Vec<String>,

  /// List of features to include.
  #[arg(short, long, value_delimiter = ',')]
  features: Vec<String>,

  /// Flavor of output font.
  #[arg(short, long)]
  flavor: Option<Flavor>,
}

impl ValueEnum for Flavor {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::Woff, Self::Woff2]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      | Self::Woff => PossibleValue::new("woff"),
      | Self::Woff2 => PossibleValue::new("woff2"),
    })
  }
}

/// Check if `path` is a font file.
fn is_font(path: &Path) -> bool {
  path.extension().map_or(false, |ext| {
    ext == "ttf" || ext == "otf" || ext == "woff" || ext == "woff2"
  })
}

fn main() {
  let args = Args::parse();

  // Get all font paths in `input` directory.
  let files = fs::read_dir("input")
    .expect("input directory should exist and be readable")
    .filter_map(Result::ok)
    .filter_map(|entry| {
      if entry.path().is_file() && is_font(&entry.path()) {
        entry.file_name().to_str().map(str::to_string)
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  if args.inspect {
    inspect::inspect(files);
  } else {
    subset::subset(files, args.subsets, args.flavor.unwrap_or_default());
  }
}
