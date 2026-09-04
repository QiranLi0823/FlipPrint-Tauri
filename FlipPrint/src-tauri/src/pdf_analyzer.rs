//! PDF 分析模块
//!
//! 使用 Python pypdf 库获取真实的页数

use serde::{Deserialize, Serialize};
use std::process::Command;

/// PDF 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfInfo {
    pub path: String,
    pub filename: String,
    pub page_count: usize,
    pub width: f32,
    pub height: f32,
    pub paper_size: String,
}

/// 分析 PDF 文件
pub fn analyze_pdf(path: &str) -> Result<PdfInfo, String> {
    // 使用 Python pypdf 获取真实的页数
    let page_count = get_pdf_page_count(path);

    if page_count == 0 {
        return Err("无法获取 PDF 页数，请确保已安装 pypdf: pip install pypdf".to_string());
    }

    // 获取纸张尺寸 (默认 A4)
    let (width, height, paper_size) = get_paper_size_default();

    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.pdf")
        .to_string();

    Ok(PdfInfo {
        path: path.to_string(),
        filename,
        page_count,
        width,
        height,
        paper_size,
    })
}

/// 使用 Python pypdf 获取 PDF 页数
fn get_pdf_page_count(path: &str) -> usize {
    let escaped_path = path.replace("\\", "\\\\").replace("'", "''");

    let script = format!(
        r#"
try:
    from pypdf import PdfReader
    reader = PdfReader(r'{}')
    print(len(reader.pages))
except Exception as e:
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
            stdout.parse().unwrap_or(0)
        }
        Err(_) => 0,
    }
}

/// 获取纸张尺寸 (默认 A4)
fn get_paper_size_default() -> (f32, f32, String) {
    (210.0, 297.0, "A4".to_string())
}
