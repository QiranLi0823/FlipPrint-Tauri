//! PDF 生成模块
//!
//! 使用 Python pypdf 提取 PDF 页面

use std::path::Path;
use std::process::Command;

/// 获取 Python 可执行文件路径
/// 优先使用 bundled Python
fn get_python_exe() -> String {
    if let Ok(exe_path) = std::env::current_exe() {
        // 尝试 exe所在目录/python/python.exe
        if let Some(exe_dir) = exe_path.parent() {
            let bundled_python = exe_dir.join("python").join("python.exe");
            if bundled_python.exists() {
                return bundled_python.to_string_lossy().to_string();
            }

            // 尝试 exe所在目录/bundle/python/python.exe
            let bundle_python = exe_dir.join("bundle").join("python").join("python.exe");
            if bundle_python.exists() {
                return bundle_python.to_string_lossy().to_string();
            }
        }
    }
    // 回退到系统 Python
    "python".to_string()
}

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

    // 检查 Python 和 pypdf
    let python_exe = get_python_exe();
    let python_check = Command::new(&python_exe)
        .args(["-B", "-c", "from pypdf import PdfReader; print('ok')"])
        .output();

    let python_available = match python_check {
        Ok(output) => String::from_utf8_lossy(&output.stdout).contains("ok"),
        Err(_) => false,
    };

    if !python_available {
        return Err("需要 Python 和 pypdf。请运行: pip install pypdf".to_string());
    }

    // 将页面数组转为 Python 列表字符串
    let pages_str = pages
        .iter()
        .map(|p| (p - 1).to_string()) // Python 是 0-based
        .collect::<Vec<_>>()
        .join(", ");
    let pages_list = format!("[{}]", pages_str);

    // 输出路径
    let temp_dir = std::env::temp_dir();
    let stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let output_path = temp_dir.join(format!("{}_print_{}.pdf", stem, side));
    let output_path_str = output_path.to_string_lossy();

    let script = format!(
        r#"
from pypdf import PdfWriter, PdfReader

reader = PdfReader(r'{}')
writer = PdfWriter()

for idx in {}:
    if idx < len(reader.pages):
        writer.add_page(reader.pages[idx])

with open(r'{}', 'wb') as f:
    writer.write(f)

print('success')
"#,
        input_path,
        pages_list,
        output_path_str
    );

    let output = Command::new(&python_exe)
        .args(["-B", "-c", &script])
        .output()
        .map_err(|e| format!("执行 Python 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if stdout != "success" {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("提取页面失败: {}", stderr));
    }

    if !output_path.exists() {
        return Err("文件生成失败".to_string());
    }

    Ok(output_path_str.to_string())
}
