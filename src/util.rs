use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use chardetng::{Iso2022JpDetection, Utf8Detection};

pub fn read_with_unknown_encoding(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let size = file.metadata().map(|m| m.len() as usize).ok();
    let mut buf = Vec::new();
    buf.try_reserve_exact(size.unwrap_or(0))?;
    file.read_to_end(&mut buf)?;

    let mut detector = chardetng::EncodingDetector::new(Iso2022JpDetection::Allow);
    detector.feed(&buf, false);
    let encoding = detector.guess(None, Utf8Detection::Allow);
    // println!("{:?}", encoding);
    Ok(encoding.decode(&buf).0.to_string())
}
