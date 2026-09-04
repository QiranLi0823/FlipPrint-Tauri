# FlipPrint

I bought an HP 105W printer, only to find it doesn't support duplex printing. So I built this tool to make manual double-sided printing actually manageable.

🪟 Windows only

---

## Features

| Feature | Description |
|---------|-------------|
| Select PDF | Drag & drop or click to select |
| Page Selection | All / Odd / Even / Invert |
| Single-Sided Print | Print selected pages directly |
| Duplex Print | Wizard: Print front → Flip → Print back |

---

## Supported Printers

Currently, FlipPrint only supports the following printers:
- HP 105
- HP 105W

> Note: Other printers may work, but have not been tested.

---

## Requirements

- **OS**: Windows 10+
- **Dependency**: [SumatraPDF](https://www.sumatrapdfreader.org/free-pdf-reader) - required for PDF printing

## Quick Start

```bash
# Install dependencies
npm install

# Run in dev mode
npm run tauri dev

# Build for production
npm run tauri build
```

---

## Project Status

### ✅ Completed
- PDF file selection (drag & drop / dialog)
- PDF analysis (page count, paper size)
- Page selection interface
- Single-sided printing
- Duplex printing wizard
- Printer selection
- Temporary file cleanup

### 🔄 In Progress
- PDF thumbnail preview

### 📋 Planned
- Print history
- Settings persistence
- macOS/Linux support
