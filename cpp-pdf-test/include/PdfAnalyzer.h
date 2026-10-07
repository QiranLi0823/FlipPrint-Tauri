/**
 * @file PdfAnalyzer.h
 * @brief PDF analysis module - read PDF file info
 */
#ifndef PDF_ANALYZER_H
#define PDF_ANALYZER_H

#include <string>
#include <podofo/podofo.h>

/**
 * @brief PDF file information struct
 */
struct FlipPdfInfo {
    std::string path;           // File path
    std::string filename;       // File name
    int page_count = 0;         // Page count
    float width = 0.0f;         // Page width (mm)
    float height = 0.0f;        // Page height (mm)
    std::string paper_size;     // Paper size name
};

/**
 * @brief PDF analyzer class
 */
class PdfAnalyzer {
public:
    /**
     * @brief Analyze PDF file
     * @param path PDF file path
     * @return FlipPdfInfo file information
     * @throws std::runtime_error if file doesn't exist or parse fails
     */
    static FlipPdfInfo analyze(const std::string& path);

    /**
     * @brief Get PDF page count
     * @param path PDF file path
     * @return Page count
     */
    static int getPageCount(const std::string& path);

    /**
     * @brief Detect paper size
     * @param width Width (points)
     * @param height Height (points)
     * @return Paper size name
     */
    static std::string detectPaperSize(double width, double height);

    /**
     * @brief Convert PoDoFo width from points to mm
     */
    static constexpr double POINTS_TO_MM = 25.4 / 72.0;
};

#endif // PDF_ANALYZER_H
