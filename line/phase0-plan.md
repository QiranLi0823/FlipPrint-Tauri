# FlipPrint 开发计划 - Phase 0 技术验证

> 本阶段目标：验证 Tauri 2 + Rust PDF 处理 + Windows 打印链路的可行性。

---

## 阶段目标

在实现漂亮 UI 之前，先用最简单的方式验证核心功能：

1. PDF 分析 - 读取页数、尺寸
2. 双面算法 - 计算打印顺序
3. PDF 生成 - 生成重排后的 PDF 文件
4. 手动打印验证 - 用户手动打印两个 PDF，验证双面效果

---

## 任务清单

### Task 1: 添加 PDF 依赖

**文件**: `FlipPrint/src-tauri/Cargo.toml`

**内容**:
```toml
[dependencies]
lopdf = "0.34"   # PDF 读取
```

**验证**: `cargo check` 不报错

---

### Task 2: 实现 PDF 分析功能

**文件**: `FlipPrint/src-tauri/src/pdf_analyzer.rs`

**功能**:
```rust
pub struct PdfInfo {
    pub page_count: usize,
    pub width: f32,
    pub height: f32,
}

pub fn analyze_pdf(path: &str) -> Result<PdfInfo, String>
```

**测试用例**:
| PDF 页数 | 预期结果 |
|---------|---------|
| 1 页 | page_count = 1, sheet_count = 1 |
| 2 页 | page_count = 2, sheet_count = 1 |
| 3 页 | page_count = 3, sheet_count = 2 |
| 5 页 | page_count = 5, sheet_count = 3 |
| 10 页 | page_count = 10, sheet_count = 5 |

---

### Task 3: 实现双面算法

**文件**: `FlipPrint/src-tauri/src/duplex.rs`

**功能**:
```rust
pub struct DuplexPlan {
    pub first_pass: Vec<usize>,   // 偶数页倒序
    pub second_pass: Vec<usize>,  // 奇数页正序
    pub sheet_count: usize,       // 纸张数
}

pub fn calculate_duplex(page_count: usize) -> DuplexPlan
```

**算法逻辑**:

对于 N 页 PDF：
- **第一遍** (偶数页倒序): `N-1, N-3, ..., 4, 2`
- **第二遍** (奇数页正序): `1, 3, 5, ..., N`
- **纸张数**: `(N + 1) / 2` (向上取整)

**测试用例**:

| N | first_pass | second_pass | sheet_count |
|---|------------|-------------|-------------|
| 1 | `[]` | `[1]` | 1 |
| 2 | `[2]` | `[1]` | 1 |
| 3 | `[2]` | `[1, 3]` | 2 |
| 4 | `[4, 2]` | `[1, 3]` | 2 |
| 5 | `[4, 2]` | `[1, 3, 5]` | 3 |
| 6 | `[6, 4, 2]` | `[1, 3, 5]` | 3 |
| 8 | `[8, 6, 4, 2]` | `[1, 3, 5, 7]` | 4 |

---

### Task 4: 实现 PDF 生成功能

**文件**: `FlipPrint/src-tauri/src/pdf_generator.rs`

**功能**:
```rust
pub fn generate_duplex_pdf(
    input_path: &str,
    output_path: &str,
    pages: &[usize],  // 页面顺序
) -> Result<(), String>
```

**逻辑**:
1. 打开原 PDF
2. 按指定顺序提取页面
3. 保存为新 PDF

**输出文件命名**:
- 第一遍: `{原文件名}_first.pdf`
- 第二遍: `{原文件名}_second.pdf`

---

### Task 5: 暴露 Tauri 命令

**文件**: `FlipPrint/src-tauri/src/lib.rs`

**命令**:
```rust
#[tauri::command]
fn analyze_pdf(path: String) -> Result<PdfInfo, String>

#[tauri::command]
fn calculate_duplex(page_count: usize) -> DuplexPlan

#[tauri::command]
fn generate_duplex_files(input_path: String) -> Result<DuplexResult, String>
```

**返回值**:
```rust
struct DuplexResult {
    pub info: PdfInfo,
    pub plan: DuplexPlan,
    pub first_file: String,  // 第一遍 PDF 路径
    pub second_file: String, // 第二遍 PDF 路径
}
```

---

### Task 6: 前端调用测试

**文件**: `FlipPrint/src/App.vue`

**测试流程**:
1. 硬编码一个测试 PDF 路径
2. 调用 `analyze_pdf` 获取信息
3. 调用 `generate_duplex_files` 生成文件
4. 在控制台打印结果

**验证**: 生成的两个 PDF 文件可以用 Windows 打印预览查看

---

### Task 7: 手动打印验证

**测试 PDF**: 找一个 3-5 页的测试 PDF

**验证步骤**:
1. 打印 `*_first.pdf`（偶数页倒序）
2. 按提示翻转纸张
3. 打印 `*_second.pdf`（奇数页正序）
4. 检查双面效果是否正确

**检查清单**:
- [ ] 页码顺序是否正确（翻页后阅读顺序正确）
- [ ] 纸张数量是否正确
- [ ] 是否有空白页

---

## 文件结构

```
FlipPrint/src-tauri/
├── src/
│   ├── main.rs
│   ├── lib.rs              # Tauri 入口
│   ├── pdf_analyzer.rs     # Task 2
│   ├── duplex.rs           # Task 3
│   └── pdf_generator.rs    # Task 4
└── Cargo.toml              # Task 1
```

---

## 风险评估

| 风险 | 可能性 | 影响 | 应对 |
|-----|-------|-----|-----|
| lopdf 库不支持某些 PDF | 低 | 高 | 换用 pdf crate 或其他库 |
| Windows 路径处理问题 | 中 | 中 | 使用 PathBuf，注意反斜杠 |
| 页面尺寸/方向问题 | 中 | 中 | 先假设 A4，后续支持其他尺寸 |

---

## 成功标准

Phase 0 完成的标志：

1. ✅ `analyze_pdf` 能正确读取任意 PDF 的页数
2. ✅ `calculate_duplex` 对所有页数都能计算出正确的打印顺序
3. ✅ `generate_duplex_files` 能生成可打印的两个 PDF 文件
4. ✅ 手动打印测试通过，双面效果正确

---

## 后续阶段预览

Phase 0 验证通过后，可以继续：

- **Phase 1**: 漂亮 UI（拖拽、打印向导）
- **Phase 2**: Windows 打印 API 集成
- **Phase 3**: 打印机 Profile 管理
- **Phase 4**: 翻纸动画和引导

---

> **核心原则**: 先跑通核心链路，再完善用户体验。
