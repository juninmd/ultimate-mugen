use crate::cli::Args;
use ume_charforge::spritekit::{
    action_text, parse_color, slice_sheet, write_variants, AxisMode, GridOptions, MaskOptions, SliceMode, SliceOptions,
};
use std::path::Path;

pub fn slice(args: &Args) -> Result<String, String> {
    let sheet = args.required(0, "sheet path")?;
    let out = args.required(1, "output folder")?;
    let mode = match args.value("mode").unwrap_or("auto") {
        "grid" => {
            let cell = args.values("cell");
            let grid = args.values("grid");
            let parse = |v: Option<&String>| v.and_then(|s| s.parse::<usize>().ok());
            SliceMode::Grid(GridOptions { cell_width: parse(cell.first()), cell_height: parse(cell.get(1)), columns: parse(grid.first()), rows: parse(grid.get(1)) })
        }
        "auto" => SliceMode::Auto { merge_distance: args.number("merge", 4)? },
        other => return Err(format!("--mode must be auto or grid, got {other}")),
    };
    let options = SliceOptions {
        mode,
        group: args.number("group", 0)?,
        start_item: args.number("start-item", 0)?,
        axis: AxisMode::parse(args.value("axis").unwrap_or("feet"))?,
        mask: MaskOptions {
            background: args.value("background").map(parse_color).transpose()?,
            tolerance: args.number("tolerance", 8)?,
            ..MaskOptions::default()
        },
        max_colors: 256,
    };
    let manifest = slice_sheet(Path::new(sheet), Path::new(out), &options)?;
    Ok(format!("sprites: {}, exact palette: {}", manifest["sprites"].as_array().map(Vec::len).unwrap_or(0), manifest["exact_palette"]))
}

pub fn palettes(args: &Args) -> Result<String, String> {
    let keep: Vec<usize> = args.values("keep").iter().filter_map(|v| v.parse().ok()).collect();
    let written = write_variants(Path::new(args.required(0, "base .act")?), Path::new(args.required(1, "output folder")?), args.values("only"), &keep)?;
    Ok(written.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n"))
}

pub fn air(args: &Args) -> Result<String, String> {
    let action = args.number("action", -1i64)?;
    if action < 0 {
        return Err("--action <number> is required".into());
    }
    action_text(Path::new(args.required(0, "manifest.json")?), action, args.number("ticks", 6)?, args.has("attack"), args.number("zones", 3)?)
}

pub fn moves(args: &Args) -> Result<String, String> {
    let built = ume_charforge::movekit::build_from_spec(Path::new(args.required(0, "moves.toml")?), Path::new(args.required(1, "output folder")?))?;
    Ok(format!(
        "actions: {}, states: {}, commands: {}",
        built.air.matches("[Begin Action").count(),
        built.cns.matches("[Statedef").count(),
        built.cmd.matches("[Command]").count()
    ))
}

pub fn check(args: &Args) -> Result<String, String> {
    let (mut lines, mut problems) = (Vec::new(), 0);
    for dir in &args.positional {
        let report = ume_charforge::check::check_directory(Path::new(dir));
        problems += report.failed + if args.has("strict") { report.unsupported } else { 0 };
        lines.extend(report.lines);
    }
    if args.positional.is_empty() {
        return Err("give at least one character folder".into());
    }
    let text = lines.join("\n");
    if problems > 0 {
        return Err(format!("{text}\n{problems} problem(s) found"));
    }
    Ok(text)
}
