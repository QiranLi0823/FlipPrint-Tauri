mod duplex;
mod pdf_analyzer;
mod pdf_generator;

use duplex::{calculate_duplex, format_page_order, DuplexPlan};
use pdf_analyzer::{analyze_pdf, PdfInfo};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplexResult {
    pub info: PdfInfo,
    pub plan: DuplexPlan,
    pub first_file: String,
    pub second_file: String,
    pub first_order: String,
    pub second_order: String,
}

#[derive(Debug, Serialize)]
pub struct PdfDataResult {
    pub data: String,  // Base64 encoded PDF data
    pub filename: String,
}

#[tauri::command]
fn cmd_read_pdf_data(path: String) -> Result<PdfDataResult, String> {
    use std::fs;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err(format!("文件不存在: {}", path));
    }

    let data = fs::read(&path_buf)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let base64_data = STANDARD.encode(&data);

    let filename = path_buf.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document.pdf")
        .to_string();

    Ok(PdfDataResult {
        data: base64_data,
        filename,
    })
}

#[tauri::command]
fn cmd_analyze_pdf(path: String) -> Result<PdfInfo, String> {
    analyze_pdf(&path)
}

#[tauri::command]
fn cmd_calculate_duplex(page_count: usize) -> DuplexPlan {
    calculate_duplex(page_count)
}

#[tauri::command]
fn cmd_generate_duplex(input_path: String) -> Result<DuplexResult, String> {
    let info = analyze_pdf(&input_path)?;
    let plan = calculate_duplex(info.page_count);
    let first_order = format_page_order(&plan.first_pass);
    let second_order = format_page_order(&plan.second_pass);

    let input_path_buf = PathBuf::from(&input_path);
    let stem = input_path_buf.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string();
    let parent = input_path_buf.parent().unwrap_or(Path::new("."));

    let first_file = parent.join(format!("{}_first.pdf", stem));
    let second_file = parent.join(format!("{}_second.pdf", stem));

    Ok(DuplexResult {
        info,
        plan,
        first_file: first_file.to_string_lossy().to_string(),
        second_file: second_file.to_string_lossy().to_string(),
        first_order,
        second_order,
    })
}

#[tauri::command]
fn cmd_get_default_printer() -> String {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .args(["-Command", "Get-CimInstance Win32_Printer | Where-Object {$_.Default -eq $true} | Select-Object -ExpandProperty Name"])
            .output();

        if let Ok(output) = output {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }
    "默认打印机".to_string()
}

#[tauri::command]
fn cmd_get_printers() -> Vec<String> {
    let mut printers = Vec::new();

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .args(["-Command", "Get-CimInstance Win32_Printer | Select-Object -ExpandProperty Name"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    printers.push(line.to_string());
                }
            }
        }
    }

    if printers.is_empty() {
        printers.push("默认打印机".to_string());
    }

    printers
}

#[tauri::command]
fn cmd_print_pdf(file_path: String, printer_name: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        // 获取 SumatraPDF 路径
        let sumatra_path = std::env::var("LOCALAPPDATA")
            .map(|p| format!("{}\\SumatraPDF\\SumatraPDF.exe", p))
            .unwrap_or_default();

        let sumatra_path = if std::path::Path::new(&sumatra_path).exists() {
            sumatra_path
        } else {
            std::env::var("ProgramFiles")
                .map(|p| format!("{}\\SumatraPDF\\SumatraPDF.exe", p))
                .unwrap_or_else(|_| String::new())
        };

        if !std::path::Path::new(&sumatra_path).exists() {
            return Err("SumatraPDF 未安装".to_string());
        }

        // 使用 PowerShell 调用 SumatraPDF 打印
        let script = format!(
            "& '{}' -print-to '{}' '{}'",
            sumatra_path,
            printer_name,
            file_path
        );

        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .spawn()
            .map_err(|e| format!("启动打印失败: {}", e))?;

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("打印功能仅支持 Windows".to_string())
    }
}

#[tauri::command]
fn cmd_open_in_default_app(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let escaped = path.replace("'", "''");
        // 使用 explorer.exe 打开文件（会用默认程序）
        let output = Command::new("cmd")
            .args(["/C", "start", "", &escaped])
            .output();

        match output {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("打开文件失败: {}", e)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("该功能仅支持 Windows".to_string())
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractPagesResult {
    pub output_path: String,
    pub page_count: usize,
}

#[tauri::command]
fn cmd_extract_pages(input_path: String, pages: Vec<usize>) -> Result<ExtractPagesResult, String> {
    use pdf_generator::extract_pages;

    let output_path = extract_pages(&input_path, &pages)?;

    Ok(ExtractPagesResult {
        output_path,
        page_count: pages.len(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            cmd_analyze_pdf,
            cmd_calculate_duplex,
            cmd_generate_duplex,
            cmd_get_default_printer,
            cmd_get_printers,
            cmd_print_pdf,
            cmd_read_pdf_data,
            cmd_open_in_default_app,
            cmd_extract_pages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
