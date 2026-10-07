/**
 * @file PdfGenerator.cpp
 * @brief PDF generation module implementation
 */

#include "PdfGenerator.h"
#include "PdfAnalyzer.h"
#include <stdexcept>
#include <filesystem>
#include <fstream>

using namespace PoDoFo;
namespace fs = std::filesystem;

std::string PdfGenerator::extractPages(const std::string& inputPath,
                                        const std::vector<int>& pages,
                                        const std::string& outputPath) {
    if (!fs::exists(inputPath)) {
        throw std::runtime_error("File not found: " + inputPath);
    }

    if (pages.empty()) {
        throw std::runtime_error("No pages to extract");
    }

    std::string finalOutputPath = outputPath;

    std::vector<int> zeroBasedPages;
    for (int page : pages) {
        if (page < 1) {
            throw std::runtime_error("Page number must be >= 1");
        }
        zeroBasedPages.push_back(page - 1);
    }

    copyPages(inputPath, finalOutputPath, zeroBasedPages);

    return finalOutputPath;
}

void PdfGenerator::copyPages(const std::string& inputPath,
                              const std::string& outputPath,
                              const std::vector<int>& pageIndices) {
    try {
        PdfMemDocument sourceDocument;
        sourceDocument.Load(inputPath);

        unsigned int totalPages = sourceDocument.GetPages().GetCount();

        std::vector<unsigned int> keepIndices;
        for (int idx : pageIndices) {
            if (idx >= 0 && idx < static_cast<int>(totalPages)) {
                keepIndices.push_back(static_cast<unsigned>(idx));
            }
        }
        // 保持原始顺序，不要排序！
        // 移除排序：std::sort(keepIndices.begin(), keepIndices.end());
        // 注意：这里假设页面索引已经是有效的（0-based）
        keepIndices.erase(std::unique(keepIndices.begin(), keepIndices.end()), keepIndices.end());

        // Create destination document and initialize page tree
        PdfMemDocument destDocument;

        // Initialize by creating a temporary page, then remove it
        // This is needed to properly initialize the document structure
        if (destDocument.GetPages().GetCount() == 0) {
            destDocument.GetPages().CreatePage(PoDoFo::PdfPageSize::A4);
            destDocument.GetPages().RemovePageAt(0);
        }

        // Add pages in groups of consecutive pages
        if (!keepIndices.empty()) {
            size_t i = 0;
            while (i < keepIndices.size()) {
                unsigned int startIdx = keepIndices[i];
                unsigned int count = 1;

                while (i + 1 < keepIndices.size() && keepIndices[i + 1] == keepIndices[i] + 1) {
                    i++;
                    count++;
                }

                destDocument.GetPages().AppendDocumentPages(sourceDocument, startIdx, count);
                i++;
            }
        }

        destDocument.Save(outputPath);

    } catch (const PdfError& e) {
        throw std::runtime_error("Copy pages failed: error code " + std::to_string(static_cast<int>(e.GetCode())));
    }
}

void PdfGenerator::mergePdfs(const std::vector<std::string>& inputPaths,
                              const std::string& outputPath) {
    if (inputPaths.empty()) {
        throw std::runtime_error("No input files");
    }

    try {
        PdfMemDocument destDocument;

        // Initialize page tree
        if (destDocument.GetPages().GetCount() == 0) {
            destDocument.GetPages().CreatePage(PoDoFo::PdfPageSize::A4);
            destDocument.GetPages().RemovePageAt(0);
        }

        for (const auto& inputPath : inputPaths) {
            PdfMemDocument sourceDocument;
            sourceDocument.Load(inputPath);
            destDocument.GetPages().AppendDocumentPages(sourceDocument);
        }

        destDocument.Save(outputPath);

    } catch (const PdfError& e) {
        throw std::runtime_error("Merge PDF failed: error code " + std::to_string(static_cast<int>(e.GetCode())));
    }
}
