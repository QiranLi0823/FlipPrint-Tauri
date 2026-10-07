cmake_minimum_required(VERSION 3.16)
project(FlipPrintCppTest VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# 输出目录
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/build)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/build)

# 寻找 PoDoFo
find_package(podofo CONFIG REQUIRED)

# 源文件
set(SOURCES
    src/PdfAnalyzer.cpp
    src/PdfGenerator.cpp
    src/main.cpp
)

# 头文件
set(HEADERS
    include/PdfAnalyzer.h
    include/PdfGenerator.h
)

# 创建可执行文件
add_executable(podofo_test ${SOURCES} ${HEADERS})

# 链接 PoDoFo 库 (使用 vcpkg 导出的 target 名称)
target_link_libraries(podofo_test PRIVATE podofo::podofo_shared)

# 包含目录
target_include_directories(podofo_test PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)
