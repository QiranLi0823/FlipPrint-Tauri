cmake_minimum_required(VERSION 3.16)
project(FlipPrintCppTest VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# vcpkg 集成
if(DEFINED ENV{VCPKG_ROOT} AND EXISTS "$ENV{VCPKG_ROOT}/vcpkg")
    set(VCPKG_ROOT "$ENV{VCPKG_ROOT}")
elseif(DEFINED ENV{VCPKG_DEFAULT_TRIPLET} AND EXISTS "${CMAKE_SOURCE_DIR}/../vcpkg")
    set(VCPKG_ROOT "${CMAKE_SOURCE_DIR}/../vcpkg")
endif()

if(VCPKG_ROOT AND EXISTS "${VCPKG_ROOT}/scripts/buildsystems/vcpkg.cmake")
    set(CMAKE_TOOLCHAIN_FILE "${VCPKG_ROOT}/scripts/buildsystems/vcpkg.cmake" CACHE FILEPATH "vcpkg toolchain")
    message(STATUS "Using vcpkg from: ${VCPKG_ROOT}")
endif()

# 寻找 PoDoFo
find_package(podofo CONFIG REQUIRED)

# 头文件
set(HEADERS
    include/PdfAnalyzer.h
    include/PdfGenerator.h
    include/PdfDll.h
)

# ==================== DLL 项目 ====================
set(DLL_OUTPUT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/dll_output)

# DLL 源文件
set(DLL_SOURCES
    src/PdfAnalyzer.cpp
    src/PdfGenerator.cpp
    src/PdfDll.cpp
)

# 创建共享库 (DLL)
add_library(pdf_dll SHARED ${DLL_SOURCES} ${HEADERS})

# DLL 导出定义
target_compile_definitions(pdf_dll PRIVATE PDF_DLL_EXPORTS)

# 链接 PoDoFo
target_link_libraries(pdf_dll PRIVATE podofo::podofo_shared)

# 包含目录
target_include_directories(pdf_dll PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# 设置 DLL 输出目录（放在根目录的 dll_output）
set_target_properties(pdf_dll PROPERTIES
    OUTPUT_NAME "pdf_dll"
    RUNTIME_OUTPUT_DIRECTORY "${DLL_OUTPUT_DIR}"
    LIBRARY_OUTPUT_DIRECTORY "${DLL_OUTPUT_DIR}"
    ARCHIVE_OUTPUT_DIRECTORY "${DLL_OUTPUT_DIR}"
    IMPORT_PREFIX ""  # 去掉 lib 前缀
)

# 强制生成导入库 (.lib)
set_target_properties(pdf_dll PROPERTIES
    IMPORTED_LOCATION "${DLL_OUTPUT_DIR}/pdf_dll.dll"
    IMPORTED_IMPLIB "${DLL_OUTPUT_DIR}/pdf_dll.lib"
)

# ==================== EXE 项目 ====================
set(EXE_OUTPUT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/build)

# EXE 源文件
set(EXE_SOURCES
    src/PdfAnalyzer.cpp
    src/PdfGenerator.cpp
    src/main.cpp
)

# 创建可执行文件
add_executable(podofo_test ${EXE_SOURCES} ${HEADERS})

# 链接 PoDoFo 库
target_link_libraries(podofo_test PRIVATE podofo::podofo_shared)

# 包含目录
target_include_directories(podofo_test PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# 设置 EXE 输出目录
set_target_properties(podofo_test PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${EXE_OUTPUT_DIR}"
)
