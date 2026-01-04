# win_opt

A comprehensive Windows 11 optimization and maintenance tool with a modern Terminal User Interface (TUI), built in Rust.

## Overview

**win_opt** is a cross-platform compiled system utility designed to streamline Windows 11 maintenance and optimization tasks. The application provides an intuitive, keyboard-driven interface for executing common system operations, from temporary file cleanup to advanced privacy configuration.

## Features

### System Maintenance
- **Temporary Files Cleanup**: Remove system temporary files to free up disk space
- **Windows Update Cleanup**: Clean Windows Update cache and component store using DISM
- **Network Utilities**: Flush DNS cache and reset Winsock catalog
- **System Repair**: Run DISM and SFC (System File Checker) for integrity verification

### Performance Optimization
- **Advanced Optimization**:
  - Prefetch files cleanup
  - High-performance power plan activation
  - Telemetry service management (DiagTrack, SysMain)

### Privacy & Security
- **Privacy Configuration**:
  - Disable telemetry services (DiagTrack, dmwappushservice, WerSvc)
  - Disable telemetry-related scheduled tasks
  - Reduce data collection

### System Information
- Display comprehensive system information including OS, CPU, RAM, and disk usage

## Requirements

- **Operating System**: Windows 7/8/10/11 (64-bit)
- **Privileges**: Some operations require administrator rights
  - System Repair (DISM/SFC)
  - Advanced Optimization
  - Privacy Configuration

## Installation

### Pre-built Binary

Download the latest `win_opt.exe` from the [Releases](../../releases) page.

### Running the Application

1. **Standard mode**: Double-click `win_opt.exe` or run from Command Prompt/PowerShell
2. **Administrator mode** (recommended): Right-click `win_opt.exe` → "Run as administrator"

## ⚠️ Important: Antivirus False Positives

**This is a legitimate system optimization tool, but antivirus software may flag it as suspicious.**

### Why does this happen?

Windows Defender and other antivirus programs use heuristic analysis to detect potentially malicious behavior. Unfortunately, **legitimate system optimization tools** perform operations that are similar to what malware does:

- Executing system commands (`cmd`, `DISM`, `sfc`, `netsh`)
- Deleting files in system directories
- Modifying Windows services
- Changing scheduled tasks
- Accessing registry and power settings

**These are normal operations for a system maintenance tool**, but they trigger antivirus heuristics.

### Solutions

#### Option 1: Add an exception in Windows Defender (Recommended)

Run PowerShell **as Administrator** and execute:

```powershell
# Add exception by file path
Add-MpPreference -ExclusionPath "C:\path\to\win_opt.exe"

# Or add exception by process name
Add-MpPreference -ExclusionProcess "win_opt.exe"
```

Alternatively, use Windows Security GUI:
1. Open **Windows Security** → **Virus & threat protection**
2. Click **Manage settings** under "Virus & threat protection settings"
3. Scroll to **Exclusions** → Click **Add or remove exclusions**
4. Click **Add an exclusion** → **File** → Select `win_opt.exe`

#### Option 2: Build from source yourself

The safest way to ensure the binary is trustworthy is to compile it yourself:

```bash
# Clone the repository
git clone https://github.com/your-username/win_opt.git
cd win_opt

# Build optimized release binary
cargo build --release --target x86_64-pc-windows-msvc

# The executable will be at: target/x86_64-pc-windows-msvc/release/win_opt.exe
```

#### Option 3: Report false positive to Microsoft

You can help improve Windows Defender by reporting the false positive:
1. Visit: https://www.microsoft.com/en-us/wdsi/filesubmission
2. Submit `win_opt.exe` for analysis
3. Select "I believe this file is clean"

### Build optimizations to reduce false positives

This project uses the following Cargo optimizations (in `Cargo.toml`) to generate cleaner binaries:

- **`opt-level = "z"`** - Size optimization (smaller, cleaner binaries)
- **`lto = true`** - Link-time optimization for better code generation
- **`strip = true`** - Remove debug symbols and metadata
- **`panic = "abort"`** - Reduce binary size
- **`codegen-units = 1`** - Better optimization quality

### Verification

You can verify the integrity of the official releases:
- Check SHA256 hashes provided in the release notes
- Scan with multiple antivirus engines on [VirusTotal.com](https://www.virustotal.com)
- Review the complete source code (single file: `src/main.rs`)

**This is open-source software** - you can audit every line of code before running it.

## Usage

### Navigation

- **Arrow Keys** or **j/k** (Vim-style): Navigate menu items
- **Enter**: Select/execute operation
- **q** or **Esc**: Exit application or return to main menu

### Menu Options

1. **Limpieza de Archivos Temporales** - Clean temporary files
2. **Limpieza de Windows Update** - Clean Windows Update cache
3. **Limpieza de Red** - Flush DNS and reset Winsock
4. **Reparación del Sistema** - Run system integrity checks
5. **Optimización Avanzada** - Advanced system optimization
6. **Privacidad y Telemetría** - Privacy and telemetry configuration
7. **Información del Sistema** - Display system information
8. **Salir** - Exit application

## Building from Source

### Prerequisites

- Rust toolchain (edition 2024 or later)
- Cargo package manager

### Dependencies

```toml
ratatui = "0.29"   # Terminal UI framework
crossterm = "0.28" # Cross-platform terminal manipulation
sysinfo = "0.30"   # System information gathering
```

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run

# Run tests
cargo test

# Lint code
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Optimized Release Build (Recommended)

Use the provided build scripts for maximum optimization and minimal antivirus false positives:

#### From Linux/macOS:

```bash
# Hacer ejecutable el script (solo la primera vez)
chmod +x build_release.sh

# Build optimizado
./build_release.sh

# Con instalación automática de dependencias
./build_release.sh --install-deps

# Con limpieza previa
./build_release.sh --clean
```

#### From Windows (PowerShell):

```powershell
# Build optimizado
.\build_release.ps1

# Con instalación automática de dependencias
.\build_release.ps1 -InstallDeps

# Con limpieza previa
.\build_release.ps1 -Clean

# Firmar digitalmente (si tienes certificado)
.\build_release.ps1 -Sign -CertPath .\cert.pfx -CertPassword "tu_password"

# Ver todas las opciones
.\build_release.ps1 -Help
```

These scripts:
- ✅ Use optimized compiler flags (`opt-level=z`, `lto`, `strip`)
- ✅ Generate smaller, cleaner binaries (~30-40% reduction)
- ✅ Cross-compile for Windows (GNU from Linux, MSVC from Windows)
- ✅ Calculate SHA256 hash for verification
- ✅ Reduce antivirus false positives significantly
- ✅ Support code signing (Windows script only)
- ✅ Verify Windows Defender status and provide exclusion commands

### Cross-compilation for Windows (from Linux/macOS)

#### Using `cross` (recommended)

```bash
# Install cross
cargo install cross --git https://github.com/cross-rs/cross

# Build for Windows
cross build --release --target x86_64-pc-windows-gnu
```

The executable will be located at:
```
target/x86_64-pc-windows-gnu/release/win_opt.exe
```

#### Using rustup + mingw-w64

```bash
# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Install MinGW-w64 toolchain (Ubuntu/Debian)
sudo apt-get install mingw-w64

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

## Technical Details

### Architecture

**win_opt** follows a single-binary architecture with all code in `src/main.rs`. The application uses:

- **Event-driven TUI**: Poll-based event loop for keyboard input handling
- **State machine**: View enum managing application navigation states
- **Widget system**: ratatui components (Block, Paragraph, List, Gauge)
- **Windows API integration**: System commands via `std::process::Command`

### Color Scheme

The interface uses a custom RGB color palette inspired by Tailwind CSS:

- **Brand Colors**: Indigo (primary), Purple (secondary), Pink (accent)
- **Semantic Colors**: Green (success), Amber (warning), Red (error), Blue (info)
- **UI Colors**: Slate variants for text and backgrounds

### View States

```rust
enum View {
    MainMenu,      // Main menu navigation
    Clean,         // Temporary files cleanup
    Network,       // Network utilities
    Repair,        // System repair tools
    Info,          // System information
    Optimize,      // Advanced optimization
    WindowsUpdate, // Windows Update cleanup
    Privacy,       // Privacy configuration
}
```

### Code Standards

- **Rust Edition**: 2024
- **Naming**: snake_case (functions/variables), UpperCamelCase (types), SCREAMING_SNAKE_CASE (constants)
- **Error Handling**: Result pattern with proper propagation
- **Safety**: No `unsafe` blocks, no `.unwrap()` in production code
- **Linting**: Clippy with `-D warnings` (zero warnings policy)
- **Formatting**: Standard rustfmt configuration

## Security Considerations

- The application executes Windows system commands (`cmd`, `powercfg`, `sc`, `DISM`, `sfc`)
- Some operations modify system services and scheduled tasks
- All operations are logged with color-coded feedback
- File deletion errors are handled gracefully (locked files are skipped)

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please ensure:

1. Code passes `cargo fmt --check`
2. Code passes `cargo clippy -- -D warnings`
3. All tests pass with `cargo test`
4. Follow the coding standards outlined in `CLAUDE.md`

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- Uses [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal library
- System information via [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - System information crate

## Version

Current version: **1.0.0**

See [RELEASE_NOTES.md](RELEASE_NOTES.md) for version history and changelog.
