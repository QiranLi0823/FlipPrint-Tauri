//! PDF 分析模块
//!
//! 使用 Python pypdf 分析 PDF

use serde::{Deserialize, Serialize};
use std::path::Path;
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

/// 获取 Python 可执行文件路径
/// 优先使用 bundled Python
fn get_python_exe() -> String {
    // 获取可执行文件所在目录
    if let Ok(exe_path) = std::env::current_exe() {
        eprintln!("[DEBUG] exe_path: {:?}", exe_path);

        // 尝试 exe所在目录/python/python.exe
        if let Some(exe_dir) = exe_path.parent() {
            let bundled_python = exe_dir.join("python").join("python.exe");
            eprintln!("[DEBUG] checking: {:?}", bundled_python);
            if bundled_python.exists() {
                eprintln!("[DEBUG] found bundled python!");
                return bundled_python.to_string_lossy().to_string();
            }

            // 尝试 exe所在目录/bundle/python/python.exe
            let bundle_python = exe_dir.join("bundle").join("python").join("python.exe");
            eprintln!("[DEBUG] checking: {:?}", bundle_python);
            if bundle_python.exists() {
                eprintln!("[DEBUG] found bundled python in bundle!");
                return bundle_python.to_string_lossy().to_string();
            }
        }
    }
    eprintln!("[DEBUG] using system python");
    // 回退到系统 Python
    "python".to_string()
}

/// 分析 PDF 文件
pub fn analyze_pdf(path: &str) -> Result<PdfInfo, String> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(format!("文件不存在: {}", path));
    }

    // 转义路径中的单引号
    let escaped_path = path.replace("\\", "\\\\").replace("'", "''");

    // 使用 pypdf 获取页数
    let script = format!(
        r#"
from pypdf import PdfReader
reader = PdfReader(r'{}')
print(len(reader.pages))
"#,
        escaped_path
    );

    let python_exe = get_python_exe();
    let output = Command::new(&python_exe)
        .args(["-B", "-c", &script])
        .output()
        .map_err(|e| format!("执行 Python 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    // 检查 stderr 中是否有真正的错误（不是警告）
    if stderr.contains("Error") || stderr.contains("Exception") || stderr.contains("Traceback") {
        if stderr.contains("No module") || stderr.contains("ModuleNotFoundError") || stderr.contains("ImportError") {
            return Err("pypdf 未安装。请运行: pip install pypdf".to_string());
        }
        return Err(format!("Python 错误: {}", stderr));
    }

    // 取最后一行作为页数（前面的行可能是警告）
    let last_line = stdout.lines().last().unwrap_or("").trim();

    // 检查输出是否是数字
    let page_count: usize = match last_line.parse() {
        Ok(n) => n,
        Err(_) => return Err(format!("无法解析页数: '{}'", stdout)),
    };

    if page_count == 0 {
        return Err("PDF 没有页面".to_string());
    }

    let filename = path_obj
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.pdf")
        .to_string();

    Ok(PdfInfo {
        path: path.to_string(),
        filename,
        page_count,
        width: 210.0,
        height: 297.0,
        paper_size: "A4".to_string(),
    })
}
