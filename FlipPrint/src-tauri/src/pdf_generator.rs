//! PDF 生成模块
//!
//! 使用 Python pypdf 提取 PDF 页面

use std::path::Path;
use std::process::Command;

/// 从 PDF 中提取指定页面并生成新 PDF
pub fn extract_pages(
    input_path: &str,
    pages: &[usize],
) -> Result<String, String> {
    let input = Path::new(input_path);
    if !input.exists() {
        return Err(format!("文件不存在: {}", input_path));
    }

    let escaped_path = input_path.replace("\\", "\\\\").replace("'", "''");

    // 将页面数组转为 Python 列表字符串
    let pages_str = pages
        .iter()
        .map(|p| (p - 1).to_string()) // Python 是 0-based
        .collect::<Vec<_>>()
        .join(", ");
    let pages_list = format!("[{}]", pages_str);

    // 生成输出文件名
    let stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let parent = input.parent().unwrap_or(Path::new("."));
    let output_path = parent.join(format!("{}_selected.pdf", stem));
    let escaped_output = output_path.to_string_lossy().replace("\\", "\\\\").replace("'", "''");

    let script = format!(
        r#"
try:
    from pypdf import PdfWriter, PdfReader

    reader = PdfReader(r'{}')
    writer = PdfWriter()

    for idx in {}:
        if idx < len(reader.pages):
            writer.add_page(reader.pages[idx])

    with open(r'{}', 'wb') as f:
        writer.write(f)

    print('success')
except Exception as e:
    print(f'error: {{e}}')
"#,
        escaped_path,
        pages_list,
        escaped_output
    );

    let output = Command::new("python")
        .args(["-c", &script])
        .output()
        .map_err(|e| format!("执行 Python 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if stdout.starts_with("error:") {
        return Err(stdout.replace("error: ", ""));
    }

    if stdout != "success" {
        return Err(format!("提取页面失败: {}", stdout));
    }

    Ok(output_path.to_string_lossy().to_string())
}

/// 生成双面打印的文件路径
pub fn generate_duplex_files(
    input_path: &str,
    _first_pages: &[usize],
    _second_pages: &[usize],
) -> Result<(String, String), String> {
    let input = Path::new(input_path);
    let stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let parent = input.parent().unwrap_or(Path::new("."));

    let first_file = parent.join(format!("{}_first.pdf", stem));
    let second_file = parent.join(format!("{}_second.pdf", stem));

    Ok((
        first_file.to_string_lossy().to_string(),
        second_file.to_string_lossy().to_string(),
    ))
}
