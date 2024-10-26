mod hubset;
mod inspect;
mod subset;

use std::fs;
use std::path::Path;

use clap::Parser;

use crate::subset::Flavor;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
  /// Output basic font metadata.
  #[arg(short, long, exclusive = true)]
  inspect: bool,

  /// Use HarfBuzz subsetter.
  #[arg(long, default_value_t = true, conflicts_with = "fonttools")]
  harfbuzz: bool,

  /// Use FontTools subsetter.
  #[arg(long, conflicts_with = "harfbuzz")]
  fonttools: bool,

  /// Font flavor to use. Incompatible with `hb-subset`.
  #[arg(short, long, conflicts_with = "harfbuzz")]
  flavor: Option<Flavor>,

  /// List of subsets to include.
  #[arg(short, long, required = true, value_delimiter = ',')]
  subsets: Vec<String>,
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
  } else if args.fonttools {
    subset::subset(files, args.subsets, args.flavor.unwrap_or_default());
  } else {
    hubset::subset(files, args.subsets);
  }
}
