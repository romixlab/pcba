use crate::util::read_with_unknown_encoding;
use anyhow::{Error, Result};
use pest::Parser;
use pest_derive::Parser;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use lexpr::Value;

pub struct KicadFile {
    pub datasheet_urls: Vec<String>,
}

#[derive(Parser)]
#[grammar = "grammar/lisp.pest"]
#[allow(dead_code)]
struct LispParser;

pub fn load_kicad_raw(path: &Path) -> Result<KicadFile> {
    let contents = read_with_unknown_encoding(path)?;
    // let mut file = match LispParser::parse(Rule::sexpr, &contents) {
    //     Ok(pairs) => pairs,
    //     Err(e) => return Err(Error::msg(format!("{e}"))),
    // };
    let file = lexpr::from_str(&contents)?;
    // println!("file: {:?}", file);
    let mut datasheet_urls = HashSet::new();
    extract_links(&file, &mut datasheet_urls);
    Ok(KicadFile {
        datasheet_urls: datasheet_urls.into_iter().collect(),
    })
}

fn extract_links(v: &Value, urls: &mut HashSet<String>) {
    match v {
        Value::Nil => {}
        Value::Null => {}
        Value::Bool(_) => {}
        Value::Number(_) => {}
        Value::Char(_) => {}
        Value::String(s) => {
            if s.starts_with("https://") || s.starts_with("http://") {
                urls.insert(s.to_string());
            }
        }
        Value::Symbol(_) => {}
        Value::Keyword(_) => {}
        Value::Bytes(_) => {}
        Value::Cons(cons) => {
            extract_links(&cons.car(), urls);
            extract_links(&cons.cdr(), urls);
        }
        Value::Vector(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::format::kicad::load_kicad_raw;

    #[test]
    fn can_read_kicad_schematic() {
        let path = Path::new("test_input/cannify_micro.kicad_sch");
        let file = load_kicad_raw(path).unwrap();
        println!("{} {:?}", file.datasheet_urls.len(), file.datasheet_urls);
    }
}