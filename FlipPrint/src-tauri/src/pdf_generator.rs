//! PDF 生成模块
//!
//! 使用 Python pypdf 提取 PDF 页面

use std::path::Path;
use std::process::Command;

/// 从 PDF 中提取指定页面并生成新 PDF
/// side: 1 表示第一面，2 表示第二面
pub fn extract_pages(
    input_path: &str,
    pages: &[usize],
    side: u8,
) -> Result<String, String> {
    let input = Path::new(input_path);
    if !input.exists() {
        return Err(format!("文件不存在: {}", input_path));
    }

    // 检查 pages 是否为空
    if pages.is_empty() {
        return Err("没有需要打印的页面".to_string());
    }

    // 将页面数组转为 Python 列表字符串
    let pages_str = pages
        .iter()
        .map(|p| (p - 1).to_string()) // Python 是 0-based
        .collect::<Vec<_>>()
        .join(", ");
    let pages_list = format!("[{}]", pages_str);

    // 使用临时目录存放生成的 PDF（避免中文路径问题）
    let temp_dir = std::env::temp_dir();
    let stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let output_path = temp_dir.join(format!("{}_print_{}.pdf", stem, side));

    let script = format!(
        r#"
import sys
from pypdf import PdfWriter, PdfReader

input_file = r'{}'
output_file = r'{}'
pages_idx = {}

try:
    reader = PdfReader(input_file)
    writer = PdfWriter()

    for idx in pages_idx:
        if idx < len(reader.pages):
            writer.add_page(reader.pages[idx])

    with open(output_file, 'wb') as f:
        writer.write(f)

    print('success')
except Exception as e:
    import traceback
    traceback.print_exc()
    print(f'error: {{e}}', file=sys.stderr)
"#,
        input_path,
        output_path.to_string_lossy(),
        pages_list
    );

    let output = Command::new("python")
        .args(["-c", &script])
        .output()
        .map_err(|e| format!("执行 Python 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stderr.is_empty() {
        eprintln!("[DEBUG] Python stderr: {}", stderr);
    }

    if stdout.starts_with("error:") {
        return Err(stdout.replace("error: ", ""));
    }

    if stdout != "success" {
        return Err(format!("提取页面失败: stdout={}, stderr={}", stdout, stderr));
    }

    // 验证文件是否生成
    if !output_path.exists() {
        return Err(format!("文件生成失败: {}", output_path.display()));
    }

    eprintln!("[DEBUG] 文件已生成: {}", output_path.display());

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
