use std::collections::HashMap;
use std::env;
use std::fmt::{self, Display};
use std::process::{self, Command};

use clap::ValueEnum;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

#[derive(Clone, Debug, Default, ValueEnum)]
pub(crate) enum Flavor {
  Woff,
  #[default]
  Woff2,
}

impl Display for Flavor {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      | Self::Woff => write!(f, "woff"),
      | Self::Woff2 => write!(f, "woff2"),
    }
  }
}

/// Subsets `file` using Unicode `ranges` and `flavor`.
fn subset_font(file: &str, ranges: &[&str], flavor: &Flavor) {
  let cwd = env::current_dir().expect("current directory should exist and be accessible");

  let input_path = cwd.join("input").join(file);

  let output_path = cwd
    .join("output")
    .join(file)
    .with_extension(flavor.to_string());

  let mut command = Command::new("pyftsubset");

  command
    .arg(input_path)
    .arg(format!("--unicodes={}", ranges.join(",")))
    .arg(format!("--output-file={}", output_path.display()))
    .arg("--flavor=woff2");

  if let Err(error) = command.status() {
    eprintln!("Failed to subset '{file}': {error:?}");
    process::exit(1);
  }
}

pub(crate) fn subset(files: Vec<String>, subsets: Vec<String>, flavor: Flavor) {
  // Common Unicode ranges mappings.
  let mappings: HashMap<&str, Vec<&str>> = HashMap::from([
    (
      "latin",
      vec![
        "U+0-FF",
        "U+131",
        "U+152",
        "U+153",
        "U+2BB",
        "U+2BC",
        "U+2C6",
        "U+2DA",
        "U+2DC",
        "U+300",
        "U+301",
        "U+303",
        "U+304",
        "U+308",
        "U+309",
        "U+323",
        "U+329",
        "U+2000-206F",
        "U+2074",
        "U+20AC",
        "U+2122",
        "U+2190-2193",
        "U+2212",
        "U+2215",
        "U+FEFF",
        "U+FFFD",
      ],
    ),
    (
      "latin-ext",
      vec![
        "U+0100-02AF",
        "U+0300-0301",
        "U+0303-0304",
        "U+0308-0309",
        "U+0323",
        "U+0329",
        "U+1E00-1EFF",
        "U+2020",
        "U+20A0-20AB",
        "U+20AD-20CF",
        "U+2113",
        "U+2C60-2C7F",
        "U+A720-A7FF",
      ],
    ),
    ("greek", vec!["U+0370-03FF"]),
    ("greek-ext", vec!["U+1F00-1FFF"]),
    (
      "cyrillic",
      vec![
        "U+0301",
        "U+0400-045F",
        "U+0490-0491",
        "U+04B0-04B1",
        "U+2116",
      ],
    ),
    (
      "cyrillic-ext",
      vec![
        "U+0460-052F",
        "U+1C80-1C88",
        "U+20B4",
        "U+2DE0-2DFF",
        "U+A640-A69F",
        "U+FE2E-FE2F",
      ],
    ),
    (
      "vietnamese",
      vec![
        "U+0102-0103",
        "U+0110-0111",
        "U+0128-0129",
        "U+0168-0169",
        "U+01A0-01A1",
        "U+01AF-01B0",
        "U+0300-0301",
        "U+0303-0304",
        "U+0308-0309",
        "U+0323",
        "U+0329",
        "U+1EA0-1EF9",
        "U+20AB",
      ],
    ),
  ]);

  // Get all Unicode ranges for subsets specified via `subsets` argument.
  let ranges = subsets
    .iter()
    .filter_map(|subset| mappings.get(subset.as_str().trim()))
    .flatten()
    .cloned()
    .collect::<Vec<_>>();

  // Setup progress bar.
  let progress_bar = ProgressBar::new(files.len() as u64);

  let progress_style = ProgressStyle::default_bar()
    .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
    .map(|template| template.progress_chars("##-"))
    .unwrap_or(ProgressStyle::default_bar());

  progress_bar.set_style(progress_style);

  // Iterate over fonts and subset them in parallel.
  files.par_iter().for_each(|file| {
    progress_bar.set_message(format!("Subsetting 'input/{file}'"));
    subset_font(file, &ranges, &flavor);
    progress_bar.inc(1);
  });

  // Finish progress bar.
  progress_bar.finish_with_message("Font subsetting complete.");
}
