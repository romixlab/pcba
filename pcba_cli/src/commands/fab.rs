use anyhow::{Context, Result, bail};
use pcba::board::Board;
use pcba::settings::FabSettings;
use std::path::{Path, PathBuf};

pub fn fab(
    board: &Path,
    out: Option<PathBuf>,
    settings: Option<PathBuf>,
    show: bool,
    dry_run: bool,
) -> Result<()> {
    if show {
        print!("{}", toml_defaults());
        return Ok(());
    }
    if board.extension().and_then(|e| e.to_str()) != Some("kicad_pcb") {
        bail!("{} is not a .kicad_pcb file", board.display());
    }
    let dir = board.parent().unwrap_or(Path::new("."));
    let settings_file = settings.or_else(|| Some(dir.join("pcba.toml")).filter(|p| p.exists()));
    let st = match &settings_file {
        Some(p) => FabSettings::from_toml(
            &std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?,
        )
        .map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))?,
        None => FabSettings::default(),
    };
    let stem = board
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("board")
        .to_string();
    let text =
        std::fs::read_to_string(board).with_context(|| format!("reading {}", board.display()))?;
    let b =
        Board::read(&text, &stem, &st).map_err(|e| anyhow::anyhow!("{}: {e}", board.display()))?;
    let res = pcba::fab(&b, &stem, &st);
    if !b.off_board.is_empty() {
        println!(
            "left out, outside the board outline: {}",
            b.off_board.join(" ")
        );
    }
    let out = out.unwrap_or_else(|| dir.join("fab"));
    for (side, s) in [("top", res.front), ("bottom", res.back)] {
        if let Some(s) = s {
            println!(
                "{side}: {} references inside their part, {} outside with a leader{}",
                s.inside,
                s.outside,
                if s.collisions > 0 {
                    format!(
                        ", {} overlap something (no free spot within reach)",
                        s.collisions
                    )
                } else {
                    String::new()
                }
            );
        }
    }
    println!(
        "{} fitted part groups, {} DNP, {} test points, {} drill sizes",
        res.tables.bom.len(),
        res.tables.dnp.iter().map(|r| r.qty).sum::<usize>(),
        res.tables.test_points.len(),
        res.tables.drills.len()
    );
    if dry_run {
        for f in &res.files {
            println!("would write {}", out.join(&f.name).display());
        }
        return Ok(());
    }
    std::fs::create_dir_all(&out).with_context(|| format!("creating {}", out.display()))?;
    for f in &res.files {
        let p = out.join(&f.name);
        std::fs::write(&p, &f.text).with_context(|| format!("writing {}", p.display()))?;
        println!("wrote {}", p.display());
    }
    Ok(())
}

fn toml_defaults() -> String {
    let d = FabSettings::default();
    let mut s =
        String::from("# pcba fab settings (lengths in mm on the board); every field is optional\n");
    for (k, v) in [
        ("min_text", d.min_text),
        ("max_text", d.max_text),
        ("text_fill", d.text_fill),
        ("char_width", d.char_width),
        ("callout_text", d.callout_text),
        ("callout_gap", d.callout_gap),
        ("callout_reach", d.callout_reach),
        ("margin", d.margin),
        ("outline_width", d.outline_width),
        ("body_width", d.body_width),
        ("thin_width", d.thin_width),
        ("testpoint_radius", d.testpoint_radius),
        ("drill_symbol", d.drill_symbol),
    ] {
        s.push_str(&format!("{k} = {v}\n"));
    }
    s.push_str(&format!("skip_libraries = {:?}\n", d.skip_libraries));
    s
}
