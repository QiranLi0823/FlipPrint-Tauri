/**
 * @file PdfDll.cpp
 * @brief C API implementation for PDF operations (PoDoFo)
 */

#define PDF_DLL_EXPORTS
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
        strncpy_s(g_error_msg, sizeof(g_error_msg), msg, _TRUNCATE);
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
        FlipPdfInfo cppInfo = PdfAnalyzer::analyze(std::string(path));

        strncpy_s(info->path, sizeof(info->path), path, _TRUNCATE);
        strncpy_s(info->filename, sizeof(info->filename), cppInfo.filename.c_str(), _TRUNCATE);
        info->page_count = cppInfo.page_count;
        info->width = cppInfo.width;
        info->height = cppInfo.height;
        strncpy_s(info->paper_size, sizeof(info->paper_size), cppInfo.paper_size.c_str(), _TRUNCATE);

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
        int count = PdfAnalyzer::getPageCount(std::string(path));
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
        std::string result = PdfGenerator::extractPages(std::string(input_path), pageList, std::string(output_path));
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
        FlipPdfInfo info = PdfAnalyzer::analyze(std::string(input_path));
        int pageCount = info.page_count;

        if (pageCount == 0) {
            pdf_set_error("PDF has no pages");
            return false;
        }

        std::vector<int> firstPass;
        std::vector<int> secondPass;

        for (int i = 1; i <= pageCount; i++) {
            if (i % 2 == 0) {
                firstPass.push_back(i);
            } else {
                secondPass.push_back(i);
            }
        }
        std::reverse(firstPass.begin(), firstPass.end());

        std::string inputPathStr(input_path);
        std::filesystem::path inputPathObj(inputPathStr);
        std::string stem = inputPathObj.stem().string();

        std::string firstPath = std::string(output_folder) + "\\" + stem + "_1_first.pdf";
        std::string secondPath = std::string(output_folder) + "\\" + stem + "_2_second.pdf";

        std::filesystem::create_directories(std::string(output_folder));

        if (!firstPass.empty()) {
            PdfGenerator::extractPages(std::string(input_path), firstPass, firstPath);
        }

        if (!secondPass.empty()) {
            PdfGenerator::extractPages(std::string(input_path), secondPass, secondPath);
        }

        strncpy_s(first_output, 512, firstPath.c_str(), _TRUNCATE);
        strncpy_s(second_output, 512, secondPath.c_str(), _TRUNCATE);

        pdf_set_error(nullptr);
        return true;

    } catch (const std::exception& e) {
        pdf_set_error(e.what());
        return false;
    }
}
