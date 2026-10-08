mod summary;

use summary::{summarize, MAX_LISTED};

/// Refuse files larger than this before reading them into memory.
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

fn usage_error() -> ! {
    eprintln!("usage: flatroom inspect [--json] <path>");
    std::process::exit(2);
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() < 3 || argv[1] != "inspect" {
        usage_error();
    }
    let mut as_json = false;
    let mut path: Option<&str> = None;
    for arg in &argv[2..] {
        if arg == "--json" {
            as_json = true;
        } else if arg.starts_with("--") || path.is_some() {
            usage_error();
        } else {
            path = Some(arg);
        }
    }
    let path = match path {
        Some(p) => p,
        None => usage_error(),
    };

    let size = match std::fs::metadata(path) {
        Ok(meta) => meta.len(),
        Err(_) => {
            eprintln!("no such file: {path}");
            std::process::exit(1);
        }
    };
    if size > MAX_FILE_BYTES {
        eprintln!("file too large: {path} ({size} bytes)");
        std::process::exit(1);
    }
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("no such file: {path}");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("cannot read file: {path}: {e}");
            std::process::exit(1);
        }
    };

    let report = match summarize(&bytes) {
        Ok(report) => report,
        Err(e) => {
            eprintln!("not JSON: {e}");
            std::process::exit(1);
        }
    };

    if as_json {
        let children: serde_json::Map<String, serde_json::Value> = report
            .children
            .iter()
            .map(|(name, count)| (name.clone(), serde_json::Value::from(*count as u64)))
            .collect();
        let out = serde_json::json!({
            "path": path,
            "size_bytes": report.size_bytes,
            "shape": report.shape,
            "keys": report.keys,
            "total_keys": report.total_keys,
            "children": children,
            "truncated": report.truncated,
        });
        println!("{out}");
        return;
    }

    println!("size: {} bytes", report.size_bytes);
    println!("shape: {}", report.shape);
    if report.shape == "object" {
        println!("keys ({}): {}", report.total_keys, report.keys.join(", "));
        if report.truncated {
            println!(
                "truncated: showing first {MAX_LISTED} of {} keys",
                report.total_keys
            );
        }
        // Kinds ("keys" vs "items") come from the parsed value; summarize
        // succeeded above so this re-parse is infallible in practice.
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("summarize accepted this JSON");
        if let serde_json::Value::Object(map) = &value {
            for (name, count) in &report.children {
                let kind = match &map[name.as_str()] {
                    serde_json::Value::Object(_) => "keys",
                    _ => "items",
                };
                println!("  {name}: {count} {kind}");
            }
        }
    } else if report.shape == "array" {
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("summarize accepted this JSON");
        if let serde_json::Value::Array(items) = &value {
            println!("items: {}", items.len());
        }
    }
}
