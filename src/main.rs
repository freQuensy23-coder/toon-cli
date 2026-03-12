mod encode;

use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

/// Blazing-fast JSON to TOON converter.
///
/// Reads JSON from stdin or a file and outputs compact TOON format.
/// Designed for LLM pipelines — supports Unix pipes natively.
#[derive(Parser)]
#[command(name = "toon-cli", version, about = "JSON to TOON — fast, pipe-friendly converter")]
struct Cli {
    /// Input JSON file (reads from stdin if omitted)
    input: Option<PathBuf>,

    /// Output file (writes to stdout if omitted)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let input = match read_input(&cli.input) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    let value = match parse_json(input) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: invalid JSON: {}", e);
            process::exit(1);
        }
    };

    let toon = encode::encode(&value);

    match cli.output {
        Some(path) => {
            let mut content = toon;
            content.push('\n');
            if let Err(e) = fs::write(&path, &content) {
                eprintln!("error: failed to write {}: {}", path.display(), e);
                process::exit(1);
            }
        }
        None => {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            out.write_all(toon.as_bytes()).unwrap();
            out.write_all(b"\n").unwrap();
        }
    }
}

fn read_input(path: &Option<PathBuf>) -> Result<Vec<u8>, String> {
    match path {
        Some(p) => {
            fs::read(p).map_err(|e| format!("failed to read {}: {}", p.display(), e))
        }
        None => {
            let mut buf = Vec::with_capacity(64 * 1024);
            io::stdin()
                .read_to_end(&mut buf)
                .map_err(|e| format!("failed to read stdin: {}", e))?;
            Ok(buf)
        }
    }
}

fn parse_json(mut data: Vec<u8>) -> Result<simd_json::OwnedValue, String> {
    simd_json::to_owned_value(&mut data).map_err(|e| e.to_string())
}
