mod bundle;
mod convert;
mod extra;
mod inspect;
mod mcp;
mod parity;
mod profile;
mod routes;
mod source;
mod tools;
#[cfg(test)]
mod tests;

use tools::{Args, TOOLS};

fn usage() -> String {
    let mut text = String::from("mugen2umf: M.U.G.E.N to UMF migration (CLI and MCP server)\n\nusage: mugen2umf <tool> [args]   |   mugen2umf mcp\n\n");
    for t in TOOLS {
        let params: Vec<String> = t.params.iter().map(|(k, _, req)| if *req { format!("--{k} <..>") } else { format!("[--{k} <..>]") }).collect();
        text += &format!("  {} {}\n      {}\n", t.name, params.join(" "), t.description);
    }
    text
}

fn parse(tool: &tools::Tool, rest: &[String]) -> Result<Args, String> {
    let mut args = Args::new();
    let mut positional = tool.params.iter().filter(|p| p.2).map(|p| p.0);
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        if let Some(key) = a.strip_prefix("--") {
            if !tool.params.iter().any(|p| p.0 == key) {
                return Err(format!("unknown option --{key} for {}", tool.name));
            }
            args.insert(key.into(), it.next().cloned().unwrap_or_default());
        } else if let Some(key) = positional.find(|k| !args.contains_key(*k)) {
            args.insert(key.into(), a.clone());
        } else {
            return Err(format!("unexpected argument: {a}"));
        }
    }
    Ok(args)
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = argv.first() else {
        print!("{}", usage());
        std::process::exit(2);
    };
    if cmd == "mcp" {
        mcp::serve(std::io::stdin().lock(), std::io::stdout());
        return;
    }
    if cmd == "--help" || cmd == "-h" || cmd == "help" {
        print!("{}", usage());
        return;
    }
    if cmd == "--version" {
        println!("mugen2umf {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let Some(tool) = TOOLS.iter().find(|t| t.name == cmd) else {
        eprintln!("unknown tool: {cmd}\n\n{}", usage());
        std::process::exit(2);
    };
    let result = parse(tool, &argv[1..]).and_then(|args| (tool.run)(&args));
    match result {
        Ok(text) => println!("{text}"),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    }
}
