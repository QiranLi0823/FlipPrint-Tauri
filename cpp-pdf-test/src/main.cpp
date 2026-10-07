/**
 * @file main.cpp
 * @brief PoDoFo PDF test program
 */

#include <iostream>
#include <string>
#include <vector>
#include <cstdlib>
#include <algorithm>
#include <filesystem>
#include "PdfAnalyzer.h"
#include "PdfGenerator.h"

void printSeparator() {
    std::cout << "\n" << std::string(50, '-') << "\n";
}

void cmdInfo(const std::string& path) {
    printSeparator();
    std::cout << "[Info] PDF Information\n";
    std::cout << "File: " << path << "\n";

    try {
        FlipPdfInfo info = PdfAnalyzer::analyze(path);

        std::cout << "\n";
        std::cout << "  Filename:   " << info.filename << "\n";
        std::cout << "  Page Count: " << info.page_count << "\n";
        std::cout << "  Size:       " << info.width << " x " << info.height << " mm\n";
        std::cout << "  Paper:      " << info.paper_size << "\n";

    } catch (const std::exception& e) {
        std::cout << "\n[Error] " << e.what() << "\n";
    }
}

void cmdExtract(const std::string& inputPath, const std::string& pagesStr, const std::string& outputFolder) {
    printSeparator();
    std::cout << "[Extract] Extract pages to folder\n";
    std::cout << "Input:  " << inputPath << "\n";
    std::cout << "Pages:  " << pagesStr << "\n";
    std::cout << "Output: " << outputFolder << "\n";

    try {
        // Ensure output directory exists
        std::filesystem::create_directories(outputFolder);

        std::vector<int> pages;
        size_t start = 0;
        while (start < pagesStr.length()) {
            size_t comma = pagesStr.find(',', start);
            std::string pageStr = (comma == std::string::npos)
                ? pagesStr.substr(start)
                : pagesStr.substr(start, comma - start);

            if (!pageStr.empty()) {
                pages.push_back(std::stoi(pageStr));
            }

            if (comma == std::string::npos) break;
            start = comma + 1;
        }

        if (pages.empty()) {
            std::cout << "\n[Error] No pages specified\n";
            return;
        }

        std::filesystem::path inputPathObj(inputPath);
        std::string stem = inputPathObj.stem().string();
        std::string outputPath = outputFolder + "\\" + stem + "_selected.pdf";

        std::string resultPath = PdfGenerator::extractPages(inputPath, pages, outputPath);

        std::cout << "\n[OK] Pages extracted: " << pages.size() << "\n";
        std::cout << "     Output: " << resultPath << "\n";

    } catch (const std::exception& e) {
        std::cout << "\n[Error] " << e.what() << "\n";
    }
}

void cmdSplit(const std::string& inputPath, const std::string& outputFolder) {
    printSeparator();
    std::cout << "[Split] Duplex print split\n";
    std::cout << "Input:  " << inputPath << "\n";
    std::cout << "Output: " << outputFolder << "\n";

    try {
        // Ensure output directory exists
        std::filesystem::create_directories(outputFolder);

        FlipPdfInfo info = PdfAnalyzer::analyze(inputPath);
        int pageCount = info.page_count;

        if (pageCount == 0) {
            std::cout << "\n[Error] PDF has no pages\n";
            return;
        }

        std::cout << "\nTotal pages: " << pageCount << "\n\n";

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

        std::cout << "First pass (print first):  ";
        for (size_t i = 0; i < firstPass.size(); i++) {
            if (i > 0) std::cout << ", ";
            std::cout << firstPass[i];
        }
        std::cout << "\n";

        std::cout << "Second pass (print second): ";
        for (size_t i = 0; i < secondPass.size(); i++) {
            if (i > 0) std::cout << ", ";
            std::cout << secondPass[i];
        }
        std::cout << "\n";

        std::filesystem::path inputPathObj(inputPath);
        std::string stem = inputPathObj.stem().string();

        std::string firstOutput = outputFolder + "\\" + stem + "_1_first.pdf";
        std::string secondOutput = outputFolder + "\\" + stem + "_2_second.pdf";

        std::cout << "\nExtracting first pass...\n";
        PdfGenerator::extractPages(inputPath, firstPass, firstOutput);
        std::cout << "[OK] First pass: " << firstOutput << "\n";

        std::cout << "Extracting second pass...\n";
        PdfGenerator::extractPages(inputPath, secondPass, secondOutput);
        std::cout << "[OK] Second pass: " << secondOutput << "\n";

        std::cout << "\n--- Sheet Preview ---\n";
        int sheetCount = (pageCount + 1) / 2;
        for (int sheet = 0; sheet < sheetCount; sheet++) {
            int front = pageCount - sheet * 2;
            int back = sheet * 2 + 1;

            std::cout << "  Sheet " << (sheet + 1) << ": ";
            if (front >= 1 && front <= pageCount && front % 2 == 0) {
                std::cout << front << " / ";
            } else {
                std::cout << "- / ";
            }
            if (back <= pageCount) {
                std::cout << back;
            } else {
                std::cout << "-";
            }
            std::cout << "\n";
        }

    } catch (const std::exception& e) {
        std::cout << "\n[Error] " << e.what() << "\n";
    }
}

void printUsage(const char* programName) {
    std::cout << "\n=======================================\n";
    std::cout << "       FlipPrint PoDoFo Tool\n";
    std::cout << "=======================================\n\n";

    std::cout << "Usage:\n\n";
    std::cout << "  " << programName << " info <pdf_path>\n";
    std::cout << "  " << programName << " extract <pdf_path> <pages> <output_folder>\n";
    std::cout << "  " << programName << " split <pdf_path> <output_folder>\n\n";
    std::cout << "Examples:\n";
    std::cout << "  " << programName << " info test.pdf\n";
    std::cout << "  " << programName << " extract test.pdf 1,3,5 C:\\output\n";
    std::cout << "  " << programName << " split test.pdf C:\\output\n";
}

int main(int argc, char* argv[]) {
    std::cout << "=======================================\n";
    std::cout << "       FlipPrint PoDoFo Tool\n";
    std::cout << "=======================================\n";

    if (argc < 2) {
        printUsage(argv[0]);
        return 0;
    }

    std::string command = argv[1];

    if (command == "info" && argc >= 3) {
        cmdInfo(argv[2]);
    }
    else if (command == "extract" && argc >= 5) {
        cmdExtract(argv[2], argv[3], argv[4]);
    }
    else if (command == "split" && argc >= 4) {
        cmdSplit(argv[2], argv[3]);
    }
    else {
        std::cout << "\n[Error] Unknown command or missing arguments\n\n";
        printUsage(argv[0]);
        return 1;
    }

    printSeparator();
    std::cout << "Done!\n";

    return 0;
}
