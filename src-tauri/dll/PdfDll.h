/**
 * @file PdfDll.h
 * @brief C API for PDF operations (for FFI with Rust)
 */
#ifndef PDF_DLL_H
#define PDF_DLL_H

#include <stdbool.h>

#ifdef PDF_DLL_EXPORTS
#define PDF_API __declspec(dllexport)
#else
#define PDF_API __declspec(dllimport)
#endif

#ifdef __cplusplus
extern "C" {
#endif

// PDF Info structure
typedef struct {
    char path[512];
    char filename[256];
    int page_count;
    float width;
    float height;
    char paper_size[64];
} PdfInfo;

// Error message buffer (thread-safe)
PDF_API const char* pdf_get_error(void);

// PDF Analysis
PDF_API bool pdf_analyze(const char* path, PdfInfo* info);
PDF_API int pdf_get_page_count(const char* path);

// PDF Page Extraction
// pages: array of 1-based page numbers
// page_count: number of pages to extract
// output_path: where to save the extracted PDF
// Returns true on success
PDF_API bool pdf_extract_pages(const char* input_path, const int* pages, int page_count, const char* output_path);

// PDF Split for Duplex Printing
// Splits PDF into two files: first_pass (even pages) and second_pass (odd pages)
// output_folder: directory where files will be saved
// first_output: buffer to receive first pass output path (should be at least 512 chars)
// second_output: buffer to receive second pass output path (should be at least 512 chars)
// Returns true on success
PDF_API bool pdf_split_duplex(const char* input_path, const char* output_folder,
                               char* first_output, char* second_output);

// Cleanup
PDF_API void pdf_cleanup(void);

#ifdef __cplusplus
}
#endif

#endif // PDF_DLL_H
