# Manual Duplex PDF Printer

> Windows-only lightweight desktop utility for manual duplex printing on printers without automatic duplex support.

## 1. 项目背景

部分打印机（例如 HP Laser 105w）在 Windows 打印设置中只提供单面打印。

当用户需要双面打印 PDF 时，通常需要手动进行两次打印：

1. 第一次打印一组页面；
2. 将打印好的纸张重新放入进纸器；
3. 第二次打印另一组页面；
4. 最终得到双面打印效果。

这个过程的主要痛点不是打印机本身，而是：

- 用户需要自己计算奇数页和偶数页；
- 偶数页通常需要反向打印；
- 不同页数需要不同的打印顺序；
- 奇数页 PDF 还需要处理最后一张纸的空白面；
- 第二次打印前需要重新设置打印范围和顺序；
- 不同打印机的出纸方向、翻纸方向可能不同。

因此，本项目的目标是将「人工双面打印」流程产品化。

---

# 2. 核心目标

## 2.1 用户目标

用户只需要：

```text
拖入 PDF
    ↓
选择「手动双面打印」
    ↓
打印第一面
    ↓
按照提示翻转纸张
    ↓
打印第二面
    ↓
完成
```

而不需要自己计算：

- 奇数页；
- 偶数页；
- 逆序；
- 页码范围；
- 翻纸方向。

---

# 3. 典型使用场景

以一个 5 页 PDF 为例：

```text
1
2
3
4
5
```

由于打印机只支持单面打印，需要将 5 页内容打印到 3 张纸上。

## 第一遍

打印偶数页，并按照倒序打印：

```text
4 → 2
```

打印完成后：

```text
纸张 1：4
纸张 2：2
```

## 翻纸

用户将打印好的纸张整体重新放入进纸器。

## 第二遍

打印奇数页：

```text
1 → 3 → 5
```

最终：

```text
纸张 1：4 / 1
纸张 2：2 / 3
纸张 3：5 / 空白
```

按照阅读顺序即可得到：

```text
1
2
3
4
5
```

---

# 4. 双面打印算法

对于一个共有 `N` 页的 PDF：

## 第一遍

打印偶数页，并按照倒序排列：

```text
N-1 或 N → ... → 4 → 2
```

具体需要根据纸张数量以及打印机的出纸方向确定。

典型情况下：

### N = 5

```text
4 → 2
```

### N = 6

```text
6 → 4 → 2
```

### N = 7

```text
6 → 4 → 2
```

### N = 8

```text
8 → 6 → 4 → 2
```

## 第二遍

打印奇数页，并按照正序排列：

```text
1 → 3 → 5 → ... → N
```

例如：

### N = 5

```text
1 → 3 → 5
```

### N = 8

```text
1 → 3 → 5 → 7
```

---

# 5. 一个重要的产品设计问题：打印机差异

不能简单认为所有打印机都适用同一种打印顺序。

实际效果取决于：

- 打印机出纸方向；
- 第一张纸位于纸堆顶部还是底部；
- 打印面朝上还是朝下；
- 进纸方向；
- 重新放纸时是否需要旋转 180°；
- 长边翻转还是短边翻转；
- 打印机驱动行为。

因此软件不应该仅仅提供：

> 「奇数页 / 偶数页」

而应该提供完整的：

> **Manual Duplex Printing Workflow**

---

# 6. 推荐产品流程

```text
PDF
 ↓
PDF Analyzer
 ↓
Duplex Calculator
 ↓
First Pass
 ↓
用户打印
 ↓
翻纸提示
 ↓
Second Pass
 ↓
用户打印
 ↓
完成
```

---

# 7. UI 设计

## 7.1 首页

```text
┌──────────────────────────────────┐
│                                  │
│       Manual Duplex Printer      │
│                                  │
│       Drop PDF here              │
│                                  │
│       [ Choose PDF ]             │
│                                  │
└──────────────────────────────────┘
```

支持：

- 拖拽 PDF；
- 文件选择；
- 最近使用文件。

---

## 7.2 PDF 分析

用户导入 PDF 后显示：

```text
paper.pdf

37 pages
A4
Portrait

Printer:
HP Laser 105w
```

然后：

```text
Printing Mode

○ Single-sided
● Manual duplex
```

---

# 8. 打印向导

## Step 1：打印第一面

例如 5 页 PDF：

```text
┌──────────────────────────────────┐
│  Step 1 / 3                      │
│                                  │
│  Print the first side            │
│                                  │
│  Pages                            │
│                                  │
│       4 → 2                      │
│                                  │
│  Please print these pages first. │
│                                  │
│       [ Print First Side ]       │
└──────────────────────────────────┘
```

---

## Step 2：翻转纸张

```text
┌──────────────────────────────────┐
│  Step 2 / 3                      │
│                                  │
│       ↻                          │
│                                  │
│  Flip the printed stack          │
│  and put it back into the        │
│  paper tray.                     │
│                                  │
│       [ Continue ]               │
└──────────────────────────────────┘
```

最好提供动态或静态示意图，明确告诉用户：

- 从哪一侧拿起纸张；
- 是否旋转 180°；
- 哪一面朝上；
- 哪一边先进入进纸器。

---

## Step 3：打印第二面

```text
┌──────────────────────────────────┐
│  Step 3 / 3                      │
│                                  │
│  Print the second side           │
│                                  │
│  Pages                            │
│                                  │
│       1 → 3 → 5                  │
│                                  │
│       [ Print Second Side ]      │
└──────────────────────────────────┘
```

---

# 9. 奇数页处理

当 PDF 总页数为奇数时：

```text
N = 5
```

第一遍：

```text
4 → 2
```

第二遍：

```text
1 → 3 → 5
```

最终最后一张纸只有一面有内容。

UI 应该明确提示：

> 最后一张纸只有一面有内容。

避免用户误认为少打印了一张。

---

# 10. 推荐的核心功能

## MVP

第一版只需要实现：

### PDF

- 导入 PDF；
- 获取页数；
- 获取页面尺寸；
- 生成指定页面顺序的 PDF。

### Duplex

- 自动计算第一遍页面顺序；
- 自动计算第二遍页面顺序；
- 奇数页处理；
- 翻纸提示。

### Printer

- 获取 Windows 打印机列表；
- 获取默认打印机；
- 调用系统打印。

### UI

- 拖拽 PDF；
- 打印机选择；
- 手动双面模式；
- 第一步打印；
- 翻纸提示；
- 第二步打印；
- 完成状态。

---

# 11. 推荐增加的功能

## 打印机 Profile

不同打印机保存不同的手动双面配置：

```json
{
  "printer": "HP Laser 105w",
  "output_face": "down",
  "flip": "long-edge",
  "rotate": false
}
```

Profile 可以包含：

- 出纸面；
- 出纸顺序；
- 翻纸方向；
- 是否旋转；
- 长边 / 短边翻转；
- 进纸方向。

---

# 12. 技术架构

本项目明确只支持：

```text
Windows 10 / Windows 11
```

推荐：

```text
Tauri 2
+
React
+
TypeScript
+
Rust
+
Windows API
```

整体架构：

```text
┌──────────────────────────────────────┐
│              Tauri 2                 │
│                                      │
│  ┌────────────────────────────────┐  │
│  │ Frontend                       │  │
│  │ React + TypeScript             │  │
│  │                                │  │
│  │ UI / Drag & Drop / Wizard      │  │
│  └───────────────┬────────────────┘  │
│                  │ invoke()           │
│                  ▼                    │
│  ┌────────────────────────────────┐  │
│  │ Rust Backend                   │  │
│  │                                │  │
│  │ PDF Processing                 │  │
│  │ Duplex Algorithm               │  │
│  │ Printer Management             │  │
│  │ Windows API                    │  │
│  └───────────────┬────────────────┘  │
└──────────────────┼───────────────────┘
                   │
                   ▼
          Windows Print System
                   │
                   ▼
             HP Laser 105w
```

---

# 13. 为什么选择 Tauri 2

本项目是一个轻量级 Windows Utility，不需要 Electron 的完整 Chromium Runtime。

Tauri 2 的优势：

- 应用体积较小；
- Rust 原生后端；
- 前端使用现代 Web 技术；
- 可以直接调用 Windows API；
- 适合本地文件处理；
- 适合本地 PDF 操作；
- 不需要云端服务；
- 不需要用户登录；
- 数据可以完全留在本地。

---

# 14. Rust Backend 模块设计

推荐目录：

```text
src-tauri/
│
├── src/
│   ├── main.rs
│   │
│   ├── commands/
│   │   ├── pdf.rs
│   │   ├── printer.rs
│   │   └── duplex.rs
│   │
│   ├── pdf/
│   │   ├── parser.rs
│   │   ├── renderer.rs
│   │   └── generator.rs
│   │
│   ├── printer/
│   │   ├── windows.rs
│   │   ├── spooler.rs
│   │   └── profile.rs
│   │
│   └── duplex/
│       ├── algorithm.rs
│       └── orientation.rs
│
└── Cargo.toml
```

---

# 15. PDF 模块

## `parser.rs`

负责：

- PDF 页数；
- 页面尺寸；
- 页面方向；
- PDF 基本信息。

例如：

```rust
struct PdfInfo {
    page_count: usize,
    width: f32,
    height: f32,
}
```

---

## `generator.rs`

负责按照指定页面顺序生成 PDF。

例如：

```text
Original PDF

1 2 3 4 5

        ↓

First PDF

4 2

        ↓

Second PDF

1 3 5
```

---

# 16. Duplex Algorithm

核心 API 可以设计为：

```rust
struct DuplexPlan {
    first_pass: Vec<usize>,
    second_pass: Vec<usize>,
    sheet_count: usize,
}
```

例如：

```text
N = 5

DuplexPlan {
    first_pass:  [4, 2],
    second_pass: [1, 3, 5],
    sheet_count: 3
}
```

前端可以直接显示：

```text
First Pass
4 → 2

Second Pass
1 → 3 → 5

3 sheets
```

---

# 17. Windows Printer 模块

Windows 本身提供打印系统和 Print Spooler API。

后续可以使用 Rust 的 Windows API bindings 与 Windows 打印系统交互。

需要研究：

- Printer Enumeration；
- Default Printer；
- Print Spooler；
- Print Job；
- DEVMODE；
- Printer Status；
- Job Status；
- Cancel / Pause / Resume。

---

# 18. MVP 阶段不要直接实现完整打印引擎

这是本项目非常重要的技术原则。

不要一开始就做：

```text
PDF
 ↓
自己 Rasterize
 ↓
自己生成 Printer Data
 ↓
PCL / XPS / GDI
 ↓
Print Spooler
 ↓
Printer
```

因为这样会显著增加开发复杂度。

---

# 19. 推荐的 MVP 打印方案

第一版：

```text
Original PDF
       │
       ▼
PDF Analyzer
       │
       ▼
Duplex Calculator
       │
       ├──────────────┐
       ▼              ▼
 First PDF        Second PDF
       │              │
       ▼              ▼
Windows System Printer
       │              │
       └──────┬───────┘
              ▼
          HP 105W
```

也就是说：

> 第一阶段让软件负责 PDF 页面的重新组织，而不是自己实现完整的打印驱动。

---

# 20. 第一阶段甚至可以不直接控制打印机

最简单的 MVP：

```text
PDF
 ↓
生成
first.pdf
second.pdf
```

例如：

```text
paper_duplex_first.pdf
paper_duplex_second.pdf
```

用户分别使用 Windows 默认打印功能打印。

这样可以快速验证：

1. 双面算法是否正确；
2. 翻纸逻辑是否正确；
3. 用户是否理解操作流程；
4. 不同打印机是否存在方向问题。

---

# 21. 后续再实现自动打印

MVP 验证完成后：

```text
Tauri
 ↓
Rust
 ↓
Windows Print API
 ↓
Printer
```

实现：

- 自动选择打印机；
- 自动提交 Print Job；
- 监控打印状态；
- 打印完成检测；
- 打印任务取消；
- 错误提示。

最终用户体验：

```text
拖入 PDF
     ↓
选择打印机
     ↓
开始
     ↓
自动打印第一面
     ↓
翻纸
     ↓
自动打印第二面
     ↓
完成
```

---

# 22. Tauri Command 设计

前端：

```typescript
const result = await invoke("analyze_pdf", {
    path: pdfPath
});
```

Rust：

```rust
#[tauri::command]
fn analyze_pdf(path: String) -> Result<PdfInfo, String> {
    // ...
}
```

生成双面任务：

```typescript
const plan = await invoke("generate_duplex_jobs", {
    path: pdfPath,
    mode: "manual"
});
```

返回：

```json
{
  "page_count": 5,
  "sheet_count": 3,
  "first_pass": [4, 2],
  "second_pass": [1, 3, 5],
  "first_pdf": "...",
  "second_pdf": "..."
}
```

---

# 23. 不需要 WASM

由于项目明确是：

```text
Windows-only
```

因此没有必要为了 PDF 处理引入 WASM。

推荐直接：

```text
Tauri
 ↓
Rust
 ↓
Native PDF Processing
```

而不是：

```text
React
 ↓
WASM
 ↓
Rust
 ↓
Windows
```

直接使用 Rust Native Code 会更加简单。

---

# 24. 项目开发阶段

## Phase 0 — 技术验证

目标：

> 验证 Tauri 2 + Windows + Rust + Printer 是否可行。

实现：

- 创建 Tauri 2 项目；
- Windows 编译；
- 获取打印机列表；
- 获取默认打印机；
- 测试调用 Windows 打印；
- 打印一个简单 PDF。

---

## Phase 1 — Duplex Algorithm

实现：

- PDF 页数分析；
- 奇偶页计算；
- 第一遍顺序；
- 第二遍顺序；
- 奇数页处理；
- Sheet Count。

测试：

```text
1 page
2 pages
3 pages
4 pages
5 pages
6 pages
...
100 pages
```

---

## Phase 2 — PDF Generation

实现：

```text
Original PDF
       ↓
First Pass PDF
       +
Second Pass PDF
```

验证：

- 页码顺序；
- 页面尺寸；
- 页面方向；
- A4；
- Landscape；
- Portrait。

---

## Phase 3 — UI

实现：

- Drag & Drop；
- PDF Preview；
- Printer Selector；
- Duplex Wizard；
- Flip Animation；
- Progress；
- Error State。

---

## Phase 4 — Windows Printing

实现：

- Windows Printer API；
- Print Job；
- Printer Status；
- Job Status；
- Cancel；
- Error Handling。

---

## Phase 5 — Printer Profiles

实现：

```text
HP Laser 105w
        ↓
Manual Duplex Profile
        ↓
Flip / Rotate / Output Direction
```

支持保存多个打印机配置。

---

# 25. 最终产品形态

最终希望做到：

```text
┌────────────────────────────────────┐
│       Manual Duplex Printer        │
│                                    │
│  Drop your PDF here                │
│                                    │
│             ↓                      │
│                                    │
│  paper.pdf                         │
│  127 pages                         │
│                                    │
│  Printer                           │
│  HP Laser 105w                     │
│                                    │
│  ● Manual Duplex                   │
│                                    │
│          [ Start Printing ]        │
└────────────────────────────────────┘
```

然后：

```text
Step 1

126 → 124 → 122 → ... → 2

[ Print First Side ]
```

↓

```text
Step 2

↻

Flip the stack
and reload the paper.

[ Continue ]
```

↓

```text
Step 3

1 → 3 → 5 → ... → 127

[ Print Second Side ]
```

↓

```text
✓

Printing Complete

127 pages
64 sheets
```

---

# 26. 项目核心价值

这个软件并不是：

> 一个 PDF 阅读器。

也不是：

> 一个完整的打印驱动。

而是：

> **一个将“单面打印机 + 人工翻纸”转化为简单双面打印流程的 Windows 工具。**

核心价值可以概括为：

```text
复杂的人工操作
        ↓
自动计算
        ↓
可视化引导
        ↓
简单的双面打印
```

---

# 27. MVP 最终边界

第一版只解决一个问题：

> **让不支持自动双面的 Windows 打印机，可以简单、可靠地完成 PDF 手动双面打印。**

暂时不做：

- Word；
- Excel；
- 图片；
- 云端；
- 用户账号；
- LLM；
- 在线 PDF；
- macOS；
- Linux；
- 完整打印驱动。

优先把：

```text
PDF
 ↓
Manual Duplex Algorithm
 ↓
PDF Generation
 ↓
Windows Printing
 ↓
User Guidance
```

这一条链路跑通。

---

# 28. 推荐最终技术栈

| 模块 | 技术 |
|---|---|
| Desktop Framework | Tauri 2 |
| Frontend | React + TypeScript |
| Backend | Rust |
| PDF Processing | Rust PDF Library |
| Windows API | windows-rs |
| Printing | Windows Print System / Print Spooler |
| UI | React |
| Build | Tauri CLI |
| Platform | Windows 10 / 11 |
| WASM | 不需要 |
| Cloud | 不需要 |
| LLM | 不需要 |

---

# 29. 最终架构

```text
                    ┌───────────────┐
                    │      PDF      │
                    └───────┬───────┘
                            │
                            ▼
                  ┌──────────────────┐
                  │   PDF Analyzer   │
                  └────────┬─────────┘
                           │
                           ▼
                 ┌────────────────────┐
                 │  Duplex Calculator │
                 └─────────┬──────────┘
                           │
                  ┌────────┴────────┐
                  ▼                 ▼
            First Pass          Second Pass
            4,2,...              1,3,5,...
                  │                 │
                  ▼                 ▼
             First PDF          Second PDF
                  │                 │
                  └────────┬────────┘
                           │
                           ▼
                  Windows Printer
                           │
                           ▼
                       HP 105W
```

---

# 30. 开发优先级

最高优先级：

```text
① Duplex Algorithm
② PDF Generation
③ Windows Printer Detection
④ Windows Printing
⑤ Flip Guidance UI
```

第二优先级：

```text
⑥ Printer Profile
⑦ Print Job Monitoring
⑧ Error Handling
⑨ Print Preview
⑩ Settings
```

后续：

```text
⑪ 多种打印机适配
⑫ 自动检测打印机行为
⑬ 更丰富的翻纸动画
⑭ 批量 PDF
⑮ 快捷操作
```

---

## 核心原则

> **先验证打印逻辑，再实现漂亮 UI；先解决 PDF + Windows 打印链路，再扩展打印机兼容性。**

本项目最值得优先验证的技术风险是：

**“Tauri 2 + Rust 能否稳定地在 Windows 下完成 PDF 页面重排以及调用系统打印链路，并让 HP Laser 105w 的手动双面流程可靠工作。”**
