# FlipPrint v0.1.0-beta

A manual duplex printing assistant for Windows. Simply drag in your PDF, and FlipPrint calculates the correct page order for two-sided printing—no more guessing which way to flip your pages.

🪟 Windows only

---

## Features

- **Automatic Page Ordering** — Calculates the correct even/odd page sequence for manual duplex printing
- **Page Selection** — All pages, odd pages, even pages, or custom selection
- **Step-by-Step Wizard** — Guides you through: Print front → Flip paper → Print back
- **Portable** — No installation required, just run the .exe

---

## Supported Printers

Tested on HP 105/105W. Other printers may work but are untested.

---

## Requirements

- **OS**: Windows 10+
- **Software**: SumatraPDF (for printing, auto-detected)

---

## Quick Start

### Pre-built Release

Download the portable release from the releases page:
1. Extract the ZIP
2. Run `FlipPrint.exe`

### Build from Source

```bash
# Build C++ PDF DLL (requires Visual Studio)
cd cpp-pdf-test
build_dll.bat

# Build FlipPrint
cd ..
build.bat

# Package portable release
pack_portable.bat
```

---

## How Manual Duplex Works

For a 6-page PDF, FlipPrint calculates:

| Pass | Pages to Print | Order |
|------|----------------|-------|
| 1st (Front) | 6, 4, 2 | Reverse |
| 2nd (Back) | 1, 3, 5 | Forward |

After printing the front side, flip the paper stack and load it back for the second pass. When folded, pages will read in correct order.

---

## Tech Stack

- **Frontend**: Vue 3 + Vite
- **Backend**: Tauri (Rust)
- **PDF Processing**: PoDoFo (C++ DLL via FFI)
- **Printing**: SumatraPDF

---

## License

MIT
