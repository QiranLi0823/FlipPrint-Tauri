mod duplex;
mod pdf_analyzer;
mod pdf_generator;

use duplex::{calculate_duplex, format_page_order, DuplexPlan};
use pdf_analyzer::{analyze_pdf, PdfInfo};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 双面打印结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DuplexResult {
    pub info: PdfInfo,
    pub plan: DuplexPlan,
    pub first_file: String,
    pub second_file: String,
    pub first_order: String,
    pub second_order: String,
}

/// 分析 PDF 文件
#[tauri::command]
fn cmd_analyze_pdf(path: String) -> Result<PdfInfo, String> {
    analyze_pdf(&path)
}

/// 计算双面打印计划（不生成文件）
#[tauri::command]
fn cmd_calculate_duplex(page_count: usize) -> DuplexPlan {
    calculate_duplex(page_count)
}

/// 生成双面打印文件
#[tauri::command]
fn cmd_generate_duplex(input_path: String) -> Result<DuplexResult, String> {
    // 分析 PDF
    let info = analyze_pdf(&input_path)?;

    // 计算双面计划
    let plan = calculate_duplex(info.page_count);

    // 保存页面顺序字符串
    let first_order = format_page_order(&plan.first_pass);
    let second_order = format_page_order(&plan.second_pass);

    // 生成文件路径
    let input_path_buf = PathBuf::from(&input_path);
    let stem = input_path_buf
        .file_stem()
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

/// 获取默认打印机名称
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

/// 获取打印机列表
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            cmd_analyze_pdf,
            cmd_calculate_duplex,
            cmd_generate_duplex,
            cmd_get_default_printer,
            cmd_get_printers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
