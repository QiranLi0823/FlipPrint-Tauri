//! PDF 生成模块
//!
//! 暂时不实现 PDF 生成，只返回路径

/// 生成双面打印的文件路径
pub fn generate_duplex_files(
    input_path: &str,
    _first_pages: &[usize],
    _second_pages: &[usize],
) -> Result<(String, String), String> {
    use std::path::Path;

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
