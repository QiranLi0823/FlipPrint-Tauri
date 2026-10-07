//! PDF 分析模块
//!
//! 使用 C++ DLL 通过 FFI 分析 PDF

use serde::{Deserialize, Serialize};
use crate::pdf_ffi;

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
    let raw_info = pdf_ffi::analyze_pdf(path)?;

    Ok(PdfInfo {
        path: raw_info.path,
        filename: raw_info.filename,
        page_count: raw_info.page_count,
        width: raw_info.width,
        height: raw_info.height,
        paper_size: raw_info.paper_size,
    })
}
