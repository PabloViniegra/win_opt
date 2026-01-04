# Security Policy

## Antivirus False Positives

### Why Does This Happen?

**win_opt** is a legitimate system optimization tool, but antivirus software may flag it as potentially malicious due to its functionality. This is a well-known issue with system utilities called **"false positives"**.

### What Operations Trigger Detection?

The following legitimate operations may trigger antivirus heuristics:

1. **Process Execution**
   - Running system commands via `cmd.exe`
   - Executing `DISM.exe`, `sfc.exe`, `netsh.exe`, `powercfg.exe`

2. **File Operations**
   - Deleting files in `%TEMP%` directory
   - Accessing system directories (`C:\Windows\Prefetch`)
   - Mass file deletion (cleanup operations)

3. **System Modifications**
   - Disabling Windows services (`sc config`)
   - Modifying scheduled tasks (`schtasks`)
   - Changing power settings (`powercfg`)
   - Registry access (telemetry settings)

4. **Network Operations**
   - Flushing DNS cache (`ipconfig /flushdns`)
   - Resetting Winsock (`netsh winsock reset`)

**All of these are normal, expected operations for a system maintenance tool.**

### How We Mitigate False Positives

This project implements several measures to reduce false positive rates:

#### 1. Build Optimizations (in `Cargo.toml`)

```toml
[profile.release]
opt-level = "z"      # Smallest binary size
lto = true           # Link-time optimization
strip = true         # Remove debug symbols
codegen-units = 1    # Better code generation
panic = "abort"      # Minimal panic runtime
```

**Benefits:**
- ✅ Smaller executable size (30-40% reduction)
- ✅ Cleaner machine code (fewer heuristic triggers)
- ✅ No debug information (reduces suspicion)
- ✅ Static linking (no external DLL dependencies)

#### 2. Transparent Operations

- ✅ **Open source**: All code is auditable (single file: `src/main.rs`)
- ✅ **User-visible**: All operations show real-time logs
- ✅ **Permission checks**: Warns when admin rights are required
- ✅ **Error handling**: Graceful failures (no forced operations)

#### 3. Code Signing (Optional)

For professional distribution, we support digital code signing:

```powershell
# Windows
.\build_release.ps1 -Sign -CertPath .\cert.pfx -CertPassword "password"
```

**Digital signatures drastically reduce false positives** because:
- Establishes publisher identity
- Proves code hasn't been tampered with
- Builds reputation over time with antivirus vendors

## Verification Methods

### For Users: How to Verify This Software is Safe

#### Option 1: Build from Source (Recommended)

The safest approach is to compile the code yourself:

```bash
# Clone repository
git clone https://github.com/PabloViniegra/win_opt.git
cd win_opt

# Review the code (it's a single file!)
cat src/main.rs

# Build
./build_release.sh  # Linux/macOS
# or
.\build_release.ps1  # Windows
```

#### Option 2: Verify SHA256 Hash

Compare the downloaded file's hash with the official release:

```powershell
# Windows
Get-FileHash -Algorithm SHA256 win_opt.exe

# Linux
sha256sum win_opt.exe
```

Compare the output with the hash provided in the [GitHub Release](../../releases).

#### Option 3: VirusTotal Scan

Upload the executable to [VirusTotal.com](https://www.virustotal.com) to see detection rates across 70+ antivirus engines.

**Expected Results:**
- ✅ Most modern AV engines: Clean
- ⚠️ Some heuristic engines: May flag as "Generic.Suspicious" or "PUA"
- ❌ Windows Defender: May trigger behavior detection on first run

**Note**: PUA (Potentially Unwanted Application) is not the same as malware. It's often used for system utilities.

## What To Do If Blocked by Antivirus

### Windows Defender

#### Method 1: PowerShell (Administrator)

```powershell
Add-MpPreference -ExclusionPath "C:\path\to\win_opt.exe"
```

#### Method 2: Windows Security GUI

1. Open **Windows Security** → **Virus & threat protection**
2. Click **Manage settings**
3. Scroll to **Exclusions** → **Add or remove exclusions**
4. Add `win_opt.exe`

### Other Antivirus Software

- **Norton**: Settings → Antivirus → Exclusions/Exceptions
- **Avast**: Settings → General → Exceptions
- **AVG**: Menu → Settings → Exceptions
- **Kaspersky**: Settings → Additional → Threats and Exclusions → Exclusions

### Report False Positive

Help improve antivirus detection by reporting false positives:

- **Microsoft Defender**: https://www.microsoft.com/en-us/wdsi/filesubmission
- **VirusTotal**: Submit and mark as clean
- **Vendor-specific**: Check your AV provider's website

## Security Best Practices

### For Users

1. ✅ **Always download from official sources** (GitHub releases)
2. ✅ **Verify SHA256 hashes** before running
3. ✅ **Review code** if you have technical knowledge
4. ✅ **Build from source** for maximum trust
5. ⚠️ **Run as Administrator only when necessary** (some features require it)

### For Developers/Contributors

1. ✅ **Never commit secrets** (certificates, keys, passwords)
2. ✅ **Follow Rust safety guidelines** (no `unsafe`, no `unwrap()`)
3. ✅ **Test all changes** (`cargo test`, `cargo clippy`)
4. ✅ **Document all system operations** clearly
5. ✅ **Use the build scripts** for releases

## Vulnerability Reporting

If you discover a **genuine security vulnerability** (not a false positive):

1. **DO NOT** open a public GitHub issue
2. Email: pablovpmadrid@gmail.com
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours.

## Technical Details for Antivirus Vendors

**win_opt** is a Rust-based system optimization utility for Windows. Key characteristics:

- **Language**: Rust (memory-safe, no buffer overflows)
- **Behavior**: Executes Windows built-in utilities (`DISM`, `sfc`, `netsh`, etc.)
- **Privileges**: Requests elevation only when necessary (UAC prompt)
- **Network**: No outbound connections (except DNS flush operation)
- **Persistence**: No registry modifications for autostart
- **File Operations**: Only in `%TEMP%` and with user consent
- **Open Source**: Complete source available at https://github.com/PabloViniegra/win_opt

**Detection Recommendation**: Whitelist by SHA256 hash or code signature.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

**Last Updated**: 2025-01-04
