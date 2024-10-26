use std::collections::HashSet;
use std::env;
use std::fs;

use rayon::prelude::*;
use ttf_parser::name_id;
use ttf_parser::Face;

fn get_font_features<'a>(face: &Face<'a>) -> Vec<String> {
  let tables = [face.tables().gpos, face.tables().gsub];
  let mut features = HashSet::new();

  for table in tables.into_iter().flatten() {
    table.features.into_iter().for_each(|feature| {
      features.insert(format!("{}", feature.tag));
    })
  }

  let mut features = features.into_iter().collect::<Vec<_>>();
  features.sort();

  features
}

fn get_font_family_names<'a>(face: &Face<'a>) -> Vec<String> {
  let mut names = Vec::new();

  for name in face.names() {
    if name.name_id == name_id::FULL_NAME && name.is_unicode() {
      if let Some(family_name) = name.to_string() {
        let language = name.language();

        names.push(format!(
          "{} ({}, {})",
          family_name,
          language.primary_language(),
          language.region()
        ));
      }
    }
  }

  names
}

fn get_font_ps_name<'a>(face: &Face<'a>) -> String {
  face
    .names()
    .into_iter()
    .find(|name| name.name_id == name_id::POST_SCRIPT_NAME && name.is_unicode())
    .and_then(|name| name.to_string())
    .unwrap_or("<none>".to_string())
}

fn inspect_font(file: &str) {
  let path = env::current_dir()
    .expect("current directory should exist and be accessible")
    .join("input")
    .join(file);

  let font = fs::read(path).expect("input file should exist and be readable");
  let face = Face::parse(&font, 0).expect("failed to parse font");

  let ps_name = get_font_ps_name(&face);
  let family_names = get_font_family_names(&face);
  let features = get_font_features(&face);

  println!("PostScript name: {}", ps_name);
  println!("Family names: {}", family_names.join(", "));
  println!("———");
  println!("Features: {}", features.join(", "));
  println!("Glyphs: {}", face.number_of_glyphs());
  println!("———");
  println!("Regular: {}", face.is_regular());
  println!("Italic: {}", face.is_italic());
  println!("Bold: {}", face.is_bold());
  println!("Oblique: {}", face.is_oblique());
  println!("———");
  println!("Variable: {}", face.is_variable());

  if face.is_variable() {
    println!("Variation axes:");

    for axis in face.variation_axes() {
      println!(
        "  - {} {}..{}, default {}",
        axis.tag, axis.min_value, axis.max_value, axis.def_value
      );
    }
  }

  println!("\n");
}

pub(crate) fn inspect(files: Vec<String>) {
  files.par_iter().for_each(|file| {
    inspect_font(file);
  });
}
