//! PDF 生成模块
//!
//! 使用 C++ DLL 通过 FFI 提取 PDF 页面

use std::path::Path;
use crate::pdf_ffi;

/// 从 PDF 中提取指定页面并生成新 PDF
/// side: 1 表示第一面，2 表示第二面
pub fn extract_pages(
    input_path: &str,
    pages: &[usize],
) -> Result<String, String> {
    let input = Path::new(input_path);
    if !input.exists() {
        return Err(format!("文件不存在: {}", input_path));
    }

    if pages.is_empty() {
        return Err("没有需要打印的页面".to_string());
    }

    // 检查页数是否有效
    let page_count = pdf_ffi::get_page_count(input_path)?;
    for &page in pages {
        if page == 0 || page > page_count {
            return Err(format!("页面 {} 超出范围 (总页数: {})", page, page_count));
        }
    }

    // 生成输出路径
    let temp_dir = std::env::temp_dir();
    let stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    // 简单处理：直接生成到 temp 目录
    // 注意：这里我们忽略 side 参数，因为调用方已经处理了命名
    let output_path = temp_dir.join(format!("{}_print.pdf", stem));
    let output_path_str = output_path.to_string_lossy().to_string();

    // 调用 DLL 提取页面
    pdf_ffi::extract_pages(input_path, pages, &output_path_str)?;

    if !output_path.exists() {
        return Err("文件生成失败".to_string());
    }

    Ok(output_path_str)
}
