use std::collections::HashMap;
use std::env;
use std::fs;

use hb_subset::{Blob, FontFace, SubsetInput};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use woff::version2::compress;

#[derive(Debug)]
enum UnicodeValue {
  Single(char),
  Range(char, char),
}

fn parse_unicode_ranges(ranges: &[&str]) -> Vec<UnicodeValue> {
  ranges
    .into_iter()
    .map(|s| {
      // Remove "U+" prefix and split by "-".
      let trailing = s.trim_start_matches("U+");

      if let Some((start, end)) = trailing.split_once('-') {
        // Parse as a range.
        let start = u32::from_str_radix(start, 16)
          .ok()
          .and_then(char::from_u32)
          .expect("expected valid unicode range start");

        let end = u32::from_str_radix(end, 16)
          .ok()
          .and_then(char::from_u32)
          .expect("expected valid unicode range end");

        UnicodeValue::Range(start, end)
      } else {
        // Parse as a single value.
        let value = u32::from_str_radix(trailing, 16)
          .ok()
          .and_then(char::from_u32)
          .expect("expected valid unicode value");

        UnicodeValue::Single(value)
      }
    })
    .collect()
}

fn get_mappings<'a>() -> HashMap<&'a str, Vec<&'a str>> {
  HashMap::from([
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
  ])
}

fn subset_font(file: &str, ranges: &[&str]) {
  let cwd = env::current_dir().expect("current directory should exist and be accessible");

  let input_path = cwd.join("input").join(file);
  let output_path = cwd.join("output").join(file).with_extension("woff2");

  // Read font.
  let font_bytes = fs::read(input_path).expect("input file should exist and be readable");
  let font_blob = Blob::from_bytes(&font_bytes).expect("input font file should be valid");
  let font = FontFace::new(font_blob).expect("input font blob should be parsable");

  // Create subset input to configure subsetting, e.g. Unicode ranges, flags, etc.
  let mut subset = SubsetInput::new().expect("subset input should be constructable");

  // Set Unicode ranges to retain.
  let mut unicode_set = subset.unicode_set();

  for value in parse_unicode_ranges(ranges) {
    match value {
      | UnicodeValue::Single(value) => unicode_set.insert(value),
      | UnicodeValue::Range(start, end) => {
        unicode_set.insert_range(start..=end);
      },
    }
  }

  if let Ok(subsetted_font) = subset.subset_font(&font) {
    // Deref to access underlying blob binary data.
    if let Some(woff2) = compress(
      &*subsetted_font.underlying_blob(),
      String::default(),
      8,
      true,
    ) {
      fs::write(output_path, woff2).expect("output file should be writeable");
    }
  }
}

pub(crate) fn subset(files: Vec<String>, subsets: Vec<String>) {
  let mappings = get_mappings();

  // Get all Unicode ranges for subsets specified via `subsets` argument.
  let ranges = subsets
    .iter()
    .filter_map(|subset| mappings.get(subset.as_str().trim()))
    .cloned()
    .flatten()
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
    subset_font(file, &ranges);
    progress_bar.inc(1);
  });

  // Finish progress bar.
  progress_bar.finish_with_message("Font subsetting complete.");
}
