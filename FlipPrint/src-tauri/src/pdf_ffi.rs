//! PDF FFI 模块
//!
//! 通过运行时动态加载 C++ PDF DLL

use std::ffi::{c_char, c_int, c_float};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use libloading::{Library, Symbol};

/// PDF 信息结构体 (对应 C 端的 PdfInfo)
#[repr(C)]
pub struct PdfInfoRaw {
    pub path: [c_char; 512],
    pub filename: [c_char; 256],
    pub page_count: c_int,
    pub width: c_float,
    pub height: c_float,
    pub paper_size: [c_char; 64],
}

/// PDF 信息 (Rust 友好格式)
#[derive(Debug, Clone)]
pub struct PdfInfo {
    pub path: String,
    pub filename: String,
    pub page_count: usize,
    pub width: f32,
    pub height: f32,
    pub paper_size: String,
}

impl From<PdfInfoRaw> for PdfInfo {
    fn from(raw: PdfInfoRaw) -> Self {
        PdfInfo {
            path: unsafe { std::ffi::CStr::from_ptr(raw.path.as_ptr()) }
                .to_string_lossy()
                .into_owned(),
            filename: unsafe { std::ffi::CStr::from_ptr(raw.filename.as_ptr()) }
                .to_string_lossy()
                .into_owned(),
            page_count: raw.page_count as usize,
            width: raw.width,
            height: raw.height,
            paper_size: unsafe { std::ffi::CStr::from_ptr(raw.paper_size.as_ptr()) }
                .to_string_lossy()
                .into_owned(),
        }
    }
}

// 延迟加载的 DLL 实例（全局缓存）
static PDF_LIB: OnceLock<Result<Library, String>> = OnceLock::new();

fn get_dll() -> Result<&'static Library, String> {
    PDF_LIB
        .get_or_init(|| {
            let possible_paths = [
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.join("pdf_dll.dll")))
                    .map(|p| p.to_string_lossy().into_owned()),
                Some("dll/pdf_dll.dll".to_string()),
                std::env::var("CARGO_MANIFEST_DIR")
                    .ok()
                    .map(|p| format!("{}\\dll\\pdf_dll.dll", p)),
            ];

            for dll_path in possible_paths.iter().flatten() {
                if !std::path::Path::new(dll_path).exists() {
                    continue;
                }

                match unsafe { Library::new(dll_path) } {
                    Ok(lib) => {
                        return Ok(lib);
                    }
                    Err(e) => {
                        eprintln!("Failed to load {}: {}", dll_path, e);
                    }
                }
            }

            Err(format!(
                "Failed to load PDF DLL. Tried: {:?}",
                possible_paths
            ))
        })
        .as_ref()
        .map_err(|e| e.clone())
}

fn get_error_from_dll(lib: &Library) -> Option<String> {
    unsafe {
        let func: Symbol<unsafe extern "C" fn() -> *const c_char> =
            lib.get(b"pdf_get_error").ok()?;
        let err = func();
        if err.is_null() {
            None
        } else {
            Some(
                std::ffi::CStr::from_ptr(err)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
}

/// 检查路径是否包含非ASCII字符
fn has_non_ascii(path: &str) -> bool {
    !path.chars().all(|c| c.is_ascii())
}

/// 创建临时副本，使用ASCII文件名
fn create_temp_copy(path: &Path) -> Result<PathBuf, String> {
    if !has_non_ascii(&path.to_string_lossy()) {
        return Ok(path.to_path_buf());
    }

    let temp_dir = std::env::temp_dir().join("flipprint_pdf");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_name = format!("{:x}.pdf", nanos);
    let temp_path = temp_dir.join(&temp_name);

    fs::copy(path, &temp_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    Ok(temp_path)
}

/// 分析 PDF 文件
pub fn analyze_pdf(path: &str) -> Result<PdfInfo, String> {
    let lib = get_dll()?;

    let work_path = create_temp_copy(Path::new(path))?;

    let mut info = PdfInfoRaw {
        path: [0; 512],
        filename: [0; 256],
        page_count: 0,
        width: 0.0,
        height: 0.0,
        paper_size: [0; 64],
    };

    let path_c = std::ffi::CString::new(work_path.to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;

    unsafe {
        let func: Symbol<unsafe extern "C" fn(*const c_char, *mut PdfInfoRaw) -> bool> =
            lib.get(b"pdf_analyze")
                .map_err(|e| format!("Function not found: {}", e))?;

        let result = func(path_c.as_ptr(), &mut info);

        if result {
            Ok(PdfInfo::from(info))
        } else {
            Err(get_error_from_dll(lib).unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}

/// 获取 PDF 页数
pub fn get_page_count(path: &str) -> Result<usize, String> {
    let lib = get_dll()?;

    let work_path = create_temp_copy(Path::new(path))?;
    let path_c = std::ffi::CString::new(work_path.to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;

    unsafe {
        let func: Symbol<unsafe extern "C" fn(*const c_char) -> c_int> =
            lib.get(b"pdf_get_page_count")
                .map_err(|e| format!("Function not found: {}", e))?;

        let count = func(path_c.as_ptr());

        if count < 0 {
            Err(get_error_from_dll(lib).unwrap_or_else(|| "Unknown error".to_string()))
        } else {
            Ok(count as usize)
        }
    }
}

/// 提取指定页面
pub fn extract_pages<P: AsRef<Path>>(
    input_path: &str,
    pages: &[usize],
    output_path: P,
) -> Result<String, String> {
    let lib = get_dll()?;

    let work_input_path = create_temp_copy(Path::new(input_path))?;

    let input_c = std::ffi::CString::new(work_input_path.to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;
    let output_c = std::ffi::CString::new(output_path.as_ref().to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;

    let pages_int: Vec<c_int> = pages.iter().map(|&p| p as c_int).collect();

    unsafe {
        let func: Symbol<
            unsafe extern "C" fn(*const c_char, *const c_int, c_int, *const c_char) -> bool,
        > = lib
            .get(b"pdf_extract_pages")
            .map_err(|e| format!("Function not found: {}", e))?;

        let result = func(
            input_c.as_ptr(),
            pages_int.as_ptr(),
            pages_int.len() as c_int,
            output_c.as_ptr(),
        );

        if result {
            Ok(output_path.as_ref().to_string_lossy().into_owned())
        } else {
            Err(get_error_from_dll(lib).unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}

/// 双面打印拆分
#[allow(dead_code)]
pub fn split_duplex<P: AsRef<Path>>(
    input_path: &str,
    output_folder: P,
) -> Result<(String, String), String> {
    let lib = get_dll()?;

    let work_input_path = create_temp_copy(Path::new(input_path))?;

    let input_c = std::ffi::CString::new(work_input_path.to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;
    let folder_c = std::ffi::CString::new(output_folder.as_ref().to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;

    let mut first_buf = [0u8; 512];
    let mut second_buf = [0u8; 512];

    unsafe {
        let func: Symbol<
            unsafe extern "C" fn(*const c_char, *const c_char, *mut c_char, *mut c_char)
                -> bool,
        > = lib
            .get(b"pdf_split_duplex")
            .map_err(|e| format!("Function not found: {}", e))?;

        let result = func(
            input_c.as_ptr(),
            folder_c.as_ptr(),
            first_buf.as_mut_ptr() as *mut c_char,
            second_buf.as_mut_ptr() as *mut c_char,
        );

        if result {
            let first = std::ffi::CStr::from_ptr(first_buf.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned();
            let second = std::ffi::CStr::from_ptr(second_buf.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned();
            Ok((first, second))
        } else {
            Err(get_error_from_dll(lib).unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}
