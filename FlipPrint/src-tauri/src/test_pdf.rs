use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: test_pdf <pdf_path>");
        return;
    }

    let path = &args[1];
    println!("Testing: {}", path);

    let page_count = get_pdf_pages(path);
    println!("\nResult: {} pages", page_count);
}

fn get_pdf_pages(path: &str) -> usize {
    let escaped_path = path.replace("\\", "\\\\").replace("'", "''");

    let script = format!(
        r#"
try:
    from pypdf import PdfReader
    reader = PdfReader(r'{}')
    print(len(reader.pages))
except Exception as e:
    print(f'Error: {{e}}')
    print(0)
"#,
        escaped_path
    );

    let output = Command::new("python")
        .args(["-c", &script])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("Python output: '{}'", stdout);
            stdout.parse().unwrap_or(0)
        }
        Err(e) => {
            println!("Failed to run Python: {}", e);
            0
        }
    }
}
