/**
 * @file PdfAnalyzer.cpp
 * @brief PDF analysis module implementation
 */

#include "PdfAnalyzer.h"
#include <stdexcept>
#include <filesystem>

using namespace PoDoFo;
namespace fs = std::filesystem;

FlipPdfInfo PdfAnalyzer::analyze(const std::string& path) {
    // Check if file exists
    if (!fs::exists(path)) {
        throw std::runtime_error("File not found: " + path);
    }

    FlipPdfInfo info;
    info.path = path;

    // Get filename
    info.filename = fs::path(path).filename().string();

    try {
        // Open PDF file (new API: use Load())
        PdfMemDocument document;
        document.Load(path);

        // Get page count (new API: GetPages().GetCount())
        info.page_count = static_cast<int>(document.GetPages().GetCount());

        if (info.page_count == 0) {
            throw std::runtime_error("PDF has no pages");
        }

        // Get first page dimensions (Rect members are public: Width, Height)
        PdfPage& firstPage = document.GetPages().GetPageAt(0);
        double width = firstPage.GetMediaBox().Width;
        double height = firstPage.GetMediaBox().Height;

        // Convert to mm
        info.width = static_cast<float>(width * POINTS_TO_MM);
        info.height = static_cast<float>(height * POINTS_TO_MM);

        // Detect paper size
        info.paper_size = detectPaperSize(width, height);

    } catch (const PdfError& e) {
        throw std::runtime_error("PDF parse failed: error code " + std::to_string(static_cast<int>(e.GetCode())));
    } catch (const std::exception& e) {
        throw;
    }

    return info;
}

int PdfAnalyzer::getPageCount(const std::string& path) {
    try {
        PdfMemDocument document;
        document.Load(path);
        return static_cast<int>(document.GetPages().GetCount());
    } catch (const PdfError& e) {
        throw std::runtime_error("Failed to read PDF page count: error code " + std::to_string(static_cast<int>(e.GetCode())));
    }
}

std::string PdfAnalyzer::detectPaperSize(double width, double height) {
    // A series paper sizes (in points, 1 point = 1/72 inch)
    // A4: 595.28 x 841.89 points
    // A5: 419.53 x 595.28 points
    // A6: 297.64 x 419.53 points

    const double tolerance = 5.0;

    // A4
    if (std::abs(width - 595.28) < tolerance && std::abs(height - 841.89) < tolerance) {
        return "A4";
    }
    // A5
    if (std::abs(width - 419.53) < tolerance && std::abs(height - 595.28) < tolerance) {
        return "A5";
    }
    // A6
    if (std::abs(width - 297.64) < tolerance && std::abs(height - 419.53) < tolerance) {
        return "A6";
    }
    // Letter
    if (std::abs(width - 612) < tolerance && std::abs(height - 792) < tolerance) {
        return "Letter";
    }
    // Legal
    if (std::abs(width - 612) < tolerance && std::abs(height - 1008) < tolerance) {
        return "Legal";
    }

    // Return actual dimensions
    char size[64];
    snprintf(size, sizeof(size), "%.1f x %.1f mm", width * POINTS_TO_MM, height * POINTS_TO_MM);
    return size;
}
