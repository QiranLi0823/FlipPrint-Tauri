# FlipPrint PoDoFo C++ PDF Tool

A C++ PDF processing tool using PoDoFo library, designed to replace Python dependencies in FlipPrint.

## Commands

### info - Show PDF Information
```powershell
.\build\podofo_test.exe info test.pdf
```
Output:
- Page count
- Page size (mm)
- Paper size (A4, Letter, etc.)

### extract - Extract Specific Pages
```powershell
.\build\podofo_test.exe extract <pdf_path> <pages> <output_folder>
```
Example:
```powershell
.\build\podofo_test.exe extract test.pdf 1,3,5 C:\output
```
Extracts pages 1, 3, 5 to `C:\output\test_selected.pdf`

### split - Duplex Print Split
```powershell
.\build\podofo_test.exe split <pdf_path> <output_folder>
```
Example:
```powershell
.\build\podofo_test.exe split test.pdf C:\output
```
Splits PDF into two files for manual duplex printing:
- `test_1_first.pdf` - First pass (even pages, reverse order)
- `test_2_second.pdf` - Second pass (odd pages, order)

Shows sheet preview:
```
Sheet 1: 4 / 1
Sheet 2: 2 / 3
Sheet 3: - / 5
```

## Build

### Full Build (first time or clean)
```powershell
build.bat
```

### Quick Build (incremental)
```powershell
quick_build.bat
```

## Project Structure

```
cpp-pdf-test/
├── CMakeLists.txt
├── build.bat           # Full build script
├── quick_build.bat     # Quick build script
├── README.md
├── include/
│   ├── PdfAnalyzer.h
│   └── PdfGenerator.h
├── src/
│   ├── main.cpp
│   ├── PdfAnalyzer.cpp
│   └── PdfGenerator.cpp
└── build/              # Output directory
```

## Dependencies

- PoDoFo 1.1.1 (via vcpkg)
- CMake 3.16+
- Visual Studio 2022 or MinGW
