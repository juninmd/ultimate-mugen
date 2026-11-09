mod cli;
mod commands;

const USAGE: &str = "usage: charforge <slice|palettes|air|moves|check> ...\n\
  slice <sheet.png> <out> [--mode auto|grid] [--grid COLS ROWS] [--cell W H] [--group N] [--start-item N] [--axis feet|row-feet|bottom-center|center] [--background #rrggbb] [--tolerance N]\n\
  palettes <base.act> <out> [--only red blue dark ...] [--keep INDEX ...]\n\
  air <manifest.json> --action N [--ticks N] [--attack] [--zones N]\n\
  moves <moves.toml> <out>\n\
  check <char_dir>... [--strict]";

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = raw.split_first() else {
        println!("{USAGE}");
        std::process::exit(2);
    };
    let args = cli::Args::parse(rest);
    let result = match command.as_str() {
        "slice" => commands::slice(&args),
        "palettes" => commands::palettes(&args),
        "air" => commands::air(&args),
        "moves" => commands::moves(&args),
        "check" => commands::check(&args),
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(text) => print!("{text}{}", if text.ends_with('\n') { "" } else { "\n" }),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
