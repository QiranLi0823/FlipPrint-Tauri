/**
 * @file PdfGenerator.h
 * @brief PDF generation module - extract pages to create new PDF
 */
#ifndef PDF_GENERATOR_H
#define PDF_GENERATOR_H

#include <string>
#include <vector>
#include <podofo/podofo.h>

/**
 * @brief PDF generator class
 */
class PdfGenerator {
public:
    /**
     * @brief Extract specified pages from PDF
     * @param inputPath Input PDF path
     * @param pages Pages to extract (1-based)
     * @param outputPath Output PDF path
     * @return Generated PDF file path
     * @throws std::runtime_error if extraction fails
     */
    static std::string extractPages(const std::string& inputPath,
                                     const std::vector<int>& pages,
                                     const std::string& outputPath);

    /**
     * @brief Copy pages to new document
     * @param inputPath Input PDF path
     * @param outputPath Output PDF path
     * @param pageIndices Page indices to copy (0-based)
     */
    static void copyPages(const std::string& inputPath,
                          const std::string& outputPath,
                          const std::vector<int>& pageIndices);

    /**
     * @brief Merge multiple PDFs
     * @param inputPaths Input file list
     * @param outputPath Output file path
     */
    static void mergePdfs(const std::vector<std::string>& inputPaths,
                          const std::string& outputPath);

private:
    /**
     * @brief Copy content from source page to destination page
     */
    static void copyPageContent(PoDoFo::PdfPage& srcPage, PoDoFo::PdfPage& destPage);
};

#endif // PDF_GENERATOR_H
