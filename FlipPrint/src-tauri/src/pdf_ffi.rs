//! PDF FFI 模块
//!
//! 通过运行时动态加载 C++ PDF DLL

use std::ffi::{c_char, c_int, c_float};
use std::path::Path;
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

// DLL 加载器
struct PdfLibrary {
    lib: Library,
}

impl PdfLibrary {
    fn new() -> Result<Self, String> {
        // 尝试多个可能的 DLL 路径
        let possible_paths = [
            // 相对于可执行文件
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("pdf_dll.dll")))
                .map(|p| p.to_string_lossy().into_owned()),
            // 相对于当前工作目录
            Some("dll/pdf_dll.dll".to_string()),
            // src-tauri/dll 目录（开发时）
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
                    return Ok(PdfLibrary { lib });
                }
                Err(e) => {
                    // 继续尝试下一个路径
                    eprintln!("Failed to load {}: {}", dll_path, e);
                }
            }
        }

        Err(format!(
            "Failed to load PDF DLL. Tried paths: {:?}",
            possible_paths
        ))
    }
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

/// 分析 PDF 文件
pub fn analyze_pdf(path: &str) -> Result<PdfInfo, String> {
    let lib = PdfLibrary::new()?;

    let mut info = PdfInfoRaw {
        path: [0; 512],
        filename: [0; 256],
        page_count: 0,
        width: 0.0,
        height: 0.0,
        paper_size: [0; 64],
    };

    let path_c = std::ffi::CString::new(path).map_err(|e| e.to_string())?;

    unsafe {
        let func: Symbol<unsafe extern "C" fn(*const c_char, *mut PdfInfoRaw) -> bool> =
            lib.lib.get(b"pdf_analyze")
                .map_err(|e| format!("Function not found: {}", e))?;

        let result = func(path_c.as_ptr(), &mut info);

        if result {
            Ok(PdfInfo::from(info))
        } else {
            Err(get_error_from_dll(&lib.lib).unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}

/// 获取 PDF 页数
pub fn get_page_count(path: &str) -> Result<usize, String> {
    let lib = PdfLibrary::new()?;
    let path_c = std::ffi::CString::new(path).map_err(|e| e.to_string())?;

    unsafe {
        let func: Symbol<unsafe extern "C" fn(*const c_char) -> c_int> =
            lib.lib.get(b"pdf_get_page_count")
                .map_err(|e| format!("Function not found: {}", e))?;

        let count = func(path_c.as_ptr());

        if count < 0 {
            Err(get_error_from_dll(&lib.lib).unwrap_or_else(|| "Unknown error".to_string()))
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
    let lib = PdfLibrary::new()?;
    let input_c = std::ffi::CString::new(input_path).map_err(|e| e.to_string())?;
    let output_c =
        std::ffi::CString::new(output_path.as_ref().to_string_lossy().as_ref())
            .map_err(|e| e.to_string())?;

    let pages_int: Vec<c_int> = pages.iter().map(|&p| p as c_int).collect();

    unsafe {
        let func: Symbol<
            unsafe extern "C" fn(*const c_char, *const c_int, c_int, *const c_char) -> bool,
        > = lib.lib
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
            Err(get_error_from_dll(&lib.lib).unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}

/// 双面打印拆分
#[allow(dead_code)]
pub fn split_duplex<P: AsRef<Path>>(
    input_path: &str,
    output_folder: P,
) -> Result<(String, String), String> {
    let lib = PdfLibrary::new()?;
    let input_c = std::ffi::CString::new(input_path).map_err(|e| e.to_string())?;
    let folder_c = std::ffi::CString::new(output_folder.as_ref().to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;

    let mut first_buf = [0u8; 512];
    let mut second_buf = [0u8; 512];

    unsafe {
        let func: Symbol<
            unsafe extern "C" fn(*const c_char, *const c_char, *mut c_char, *mut c_char)
                -> bool,
        > = lib.lib
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
            Err(get_error_from_dll(&lib.lib).unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}
