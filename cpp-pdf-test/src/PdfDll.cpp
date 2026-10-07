/**
 * @file PdfDll.cpp
 * @brief C API implementation for PDF operations (PoDoFo)
 */

#include "PdfDll.h"
#include "PdfAnalyzer.h"
#include "PdfGenerator.h"
#include <cstring>
#include <cstdio>
#include <vector>
#include <filesystem>

// Thread-safe error message buffer
static char g_error_msg[1024] = {0};

const char* pdf_get_error(void) {
    return g_error_msg;
}

void pdf_set_error(const char* msg) {
    if (msg) {
        strncpy_s(g_error_msg, msg, sizeof(g_error_msg) - 1);
        g_error_msg[sizeof(g_error_msg) - 1] = '\0';
    } else {
        g_error_msg[0] = '\0';
    }
}

void pdf_cleanup(void) {
    g_error_msg[0] = '\0';
}

bool pdf_analyze(const char* path, PdfInfo* info) {
    if (!path || !info) {
        pdf_set_error("Invalid parameters");
        return false;
    }

    try {
        FlipPdfInfo cppInfo = PdfAnalyzer::analyze(path);

        strncpy_s(info->path, path, sizeof(info->path) - 1);
        info->path[sizeof(info->path) - 1] = '\0';

        strncpy_s(info->filename, cppInfo.filename.c_str(), sizeof(info->filename) - 1);
        info->filename[sizeof(info->filename) - 1] = '\0';

        info->page_count = cppInfo.page_count;
        info->width = cppInfo.width;
        info->height = cppInfo.height;

        strncpy_s(info->paper_size, cppInfo.paper_size.c_str(), sizeof(info->paper_size) - 1);
        info->paper_size[sizeof(info->paper_size) - 1] = '\0';

        pdf_set_error(nullptr);
        return true;

    } catch (const std::exception& e) {
        pdf_set_error(e.what());
        return false;
    }
}

int pdf_get_page_count(const char* path) {
    if (!path) {
        pdf_set_error("Invalid path");
        return -1;
    }

    try {
        int count = PdfAnalyzer::getPageCount(path);
        pdf_set_error(nullptr);
        return count;
    } catch (const std::exception& e) {
        pdf_set_error(e.what());
        return -1;
    }
}

bool pdf_extract_pages(const char* input_path, const int* pages, int page_count, const char* output_path) {
    if (!input_path || !pages || page_count <= 0 || !output_path) {
        pdf_set_error("Invalid parameters");
        return false;
    }

    try {
        std::vector<int> pageList(pages, pages + page_count);
        std::string result = PdfGenerator::extractPages(input_path, pageList, output_path);
        pdf_set_error(nullptr);
        return true;
    } catch (const std::exception& e) {
        pdf_set_error(e.what());
        return false;
    }
}

bool pdf_split_duplex(const char* input_path, const char* output_folder, char* first_output, char* second_output) {
    if (!input_path || !output_folder || !first_output || !second_output) {
        pdf_set_error("Invalid parameters");
        return false;
    }

    try {
        // Analyze PDF to get page count
        FlipPdfInfo info = PdfAnalyzer::analyze(input_path);
        int pageCount = info.page_count;

        if (pageCount == 0) {
            pdf_set_error("PDF has no pages");
            return false;
        }

        // Calculate duplex order
        std::vector<int> firstPass;  // Even pages reverse
        std::vector<int> secondPass; // Odd pages order

        for (int i = 1; i <= pageCount; i++) {
            if (i % 2 == 0) {
                firstPass.push_back(i);
            } else {
                secondPass.push_back(i);
            }
        }
        std::reverse(firstPass.begin(), firstPass.end());

        // Get filename stem
        std::filesystem::path inputPathObj(input_path);
        std::string stem = inputPathObj.stem().string();

        // Build output paths
        std::string firstPath = std::string(output_folder) + "\\" + stem + "_1_first.pdf";
        std::string secondPath = std::string(output_folder) + "\\" + stem + "_2_second.pdf";

        // Ensure output directory exists
        std::filesystem::create_directories(output_folder);

        // Extract first pass
        if (!firstPass.empty()) {
            PdfGenerator::extractPages(input_path, firstPass, firstPath);
        }

        // Extract second pass
        if (!secondPass.empty()) {
            PdfGenerator::extractPages(input_path, secondPass, secondPath);
        }

        // Return output paths
        strncpy_s(first_output, 512, firstPath.c_str(), 512 - 1);
        first_output[511] = '\0';

        strncpy_s(second_output, 512, secondPath.c_str(), 512 - 1);
        second_output[511] = '\0';

        pdf_set_error(nullptr);
        return true;

    } catch (const std::exception& e) {
        pdf_set_error(e.what());
        return false;
    }
}
