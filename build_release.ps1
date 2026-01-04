# Script de compilación optimizada para win_opt (desde Windows)
# Reduce detecciones de falsos positivos en antivirus

param(
    [switch]$Clean = $false,
    [switch]$InstallDeps = $false,
    [switch]$Sign = $false,
    [string]$CertPath = "",
    [string]$CertPassword = "",
    [switch]$Help = $false
)

# Función para mostrar ayuda
function Show-Usage {
    Write-Host ""
    Write-Host "Uso: .\build_release.ps1 [opciones]" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Opciones:" -ForegroundColor White
    Write-Host "  -Clean           Limpiar builds anteriores" -ForegroundColor Gray
    Write-Host "  -InstallDeps     Instalar dependencias necesarias" -ForegroundColor Gray
    Write-Host "  -Sign            Firmar el ejecutable digitalmente" -ForegroundColor Gray
    Write-Host "  -CertPath        Ruta al certificado (.pfx)" -ForegroundColor Gray
    Write-Host "  -CertPassword    Contraseña del certificado" -ForegroundColor Gray
    Write-Host "  -Help            Mostrar esta ayuda" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Ejemplos:" -ForegroundColor White
    Write-Host "  .\build_release.ps1                    # Build normal" -ForegroundColor Gray
    Write-Host "  .\build_release.ps1 -Clean             # Limpiar y compilar" -ForegroundColor Gray
    Write-Host "  .\build_release.ps1 -InstallDeps       # Instalar deps primero" -ForegroundColor Gray
    Write-Host "  .\build_release.ps1 -Sign -CertPath cert.pfx -CertPassword pass123" -ForegroundColor Gray
    Write-Host ""
}

if ($Help) {
    Show-Usage
    exit 0
}

Write-Host ""
Write-Host "🔨 win_opt - Script de compilación optimizada" -ForegroundColor Cyan
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host ""

# Instalar dependencias si se solicita
if ($InstallDeps) {
    Write-Host "📦 Verificando e instalando dependencias..." -ForegroundColor Yellow
    Write-Host ""

    # Verificar Rust
    $rustInstalled = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $rustInstalled) {
        Write-Host "❌ Rust no está instalado" -ForegroundColor Red
        Write-Host "Por favor, instala Rust desde: https://rustup.rs/" -ForegroundColor Red
        Write-Host "O ejecuta este comando en PowerShell (Admin):" -ForegroundColor Yellow
        Write-Host "  Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe; .\rustup-init.exe" -ForegroundColor Gray
        exit 1
    }

    # Agregar target MSVC (preferido en Windows)
    Write-Host "📦 Agregando target x86_64-pc-windows-msvc..." -ForegroundColor Yellow
    rustup target add x86_64-pc-windows-msvc

    Write-Host ""
    Write-Host "✅ Dependencias verificadas" -ForegroundColor Green
    Write-Host ""
}

# Limpiar builds anteriores si se solicita
if ($Clean) {
    Write-Host "🧹 Limpiando builds anteriores..." -ForegroundColor Yellow
    cargo clean
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Limpieza completada" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Error durante la limpieza" -ForegroundColor Yellow
    }
    Write-Host ""
}

# Verificar que Rust esté instalado
Write-Host "🔍 Verificando instalación de Rust..." -ForegroundColor Yellow
$rustVersion = cargo --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error: Rust no está instalado o no está en PATH" -ForegroundColor Red
    Write-Host "Instala Rust desde: https://rustup.rs/" -ForegroundColor Red
    Write-Host ""
    Write-Host "Comando rápido (PowerShell Admin):" -ForegroundColor Yellow
    Write-Host "  Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe; .\rustup-init.exe" -ForegroundColor Gray
    exit 1
}
Write-Host "✅ $rustVersion" -ForegroundColor Green
Write-Host ""

# Verificar Visual Studio Build Tools (necesario para MSVC)
Write-Host "🔍 Verificando Visual Studio Build Tools..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$buildToolsInstalled = $false

if (Test-Path $vsWhere) {
    $vsInstall = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($vsInstall) {
        $buildToolsInstalled = $true
        Write-Host "✅ Visual Studio Build Tools detectado" -ForegroundColor Green
    }
}

if (-not $buildToolsInstalled) {
    Write-Host "⚠️  Visual Studio Build Tools no detectado" -ForegroundColor Yellow
    Write-Host "   Para mejor compatibilidad, instala Visual Studio Build Tools:" -ForegroundColor Gray
    Write-Host "   https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Gray
    Write-Host ""
    Write-Host "   Alternativamente, el build continuará con MinGW..." -ForegroundColor Gray
}
Write-Host ""

# Determinar target (preferir MSVC en Windows)
$target = "x86_64-pc-windows-msvc"
Write-Host "🎯 Target seleccionado: $target" -ForegroundColor Cyan

# Verificar que el target esté instalado
Write-Host "🔍 Verificando target $target..." -ForegroundColor Yellow
$targets = rustup target list --installed 2>&1
if ($targets -notmatch $target) {
    Write-Host "📦 Instalando target $target..." -ForegroundColor Yellow
    rustup target add $target
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Error al instalar target" -ForegroundColor Red
        exit 1
    }
}
Write-Host "✅ Target disponible" -ForegroundColor Green
Write-Host ""

# Compilar con optimizaciones
Write-Host "🚀 Compilando en modo release con optimizaciones..." -ForegroundColor Cyan
Write-Host "   - opt-level: z (optimización de tamaño)" -ForegroundColor Gray
Write-Host "   - lto: true (link-time optimization)" -ForegroundColor Gray
Write-Host "   - strip: true (eliminar símbolos)" -ForegroundColor Gray
Write-Host "   - codegen-units: 1 (mejor optimización)" -ForegroundColor Gray
Write-Host "   - panic: abort (reducir tamaño)" -ForegroundColor Gray
Write-Host ""

# Ejecutar compilación
$buildStartTime = Get-Date
cargo build --release --target $target

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "❌ Error durante la compilación" -ForegroundColor Red
    Write-Host "Verifica que todas las dependencias estén instaladas correctamente" -ForegroundColor Yellow
    exit 1
}

$buildEndTime = Get-Date
$buildDuration = $buildEndTime - $buildStartTime

Write-Host ""
Write-Host "✅ Compilación exitosa en $($buildDuration.TotalSeconds.ToString('0.0'))s" -ForegroundColor Green
Write-Host ""

# Ruta del ejecutable
$exePath = "target\$target\release\win_opt.exe"

# Mostrar información del binario
Write-Host "📊 Información del binario:" -ForegroundColor Cyan
Write-Host "   Ruta: $exePath" -ForegroundColor Gray

if (Test-Path $exePath) {
    $fileInfo = Get-Item $exePath
    $fileSizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
    $fileSizeKB = [math]::Round($fileInfo.Length / 1KB, 0)

    Write-Host "   Tamaño: $fileSizeMB MB ($fileSizeKB KB)" -ForegroundColor Gray

    # Calcular hash SHA256
    try {
        $hash = Get-FileHash -Path $exePath -Algorithm SHA256
        Write-Host "   SHA256: $($hash.Hash)" -ForegroundColor Gray
    } catch {
        Write-Host "   ⚠️  No se pudo calcular SHA256" -ForegroundColor Yellow
    }

    # Información adicional del archivo
    Write-Host "   Fecha: $($fileInfo.LastWriteTime)" -ForegroundColor Gray
} else {
    Write-Host "❌ Error: El ejecutable no se generó en la ruta esperada" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Firmar el ejecutable si se proporciona certificado
if ($Sign) {
    Write-Host "🔐 Firmando el ejecutable digitalmente..." -ForegroundColor Cyan
    Write-Host ""

    if (-not $CertPath) {
        Write-Host "❌ Error: Debe especificar -CertPath con la ruta al certificado .pfx" -ForegroundColor Red
        Write-Host "Ejemplo: .\build_release.ps1 -Sign -CertPath .\mi_cert.pfx -CertPassword MiPassword123" -ForegroundColor Yellow
        exit 1
    }

    if (-not (Test-Path $CertPath)) {
        Write-Host "❌ Error: Certificado no encontrado en: $CertPath" -ForegroundColor Red
        exit 1
    }

    # Buscar signtool.exe
    $signtoolPaths = @(
        "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe",
        "${env:ProgramFiles}\Windows Kits\10\bin\*\x64\signtool.exe",
        "${env:ProgramFiles(x86)}\Microsoft SDKs\Windows\*\bin\x64\signtool.exe"
    )

    $signtoolPath = $null
    foreach ($path in $signtoolPaths) {
        $found = Get-ChildItem $path -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) {
            $signtoolPath = $found.FullName
            break
        }
    }

    if (-not $signtoolPath) {
        Write-Host "❌ signtool.exe no encontrado" -ForegroundColor Red
        Write-Host "   Instala Windows SDK desde:" -ForegroundColor Yellow
        Write-Host "   https://developer.microsoft.com/windows/downloads/windows-sdk/" -ForegroundColor Gray
        exit 1
    }

    Write-Host "   Usando: $signtoolPath" -ForegroundColor Gray

    # Ejecutar firma
    $signArgs = @(
        "sign",
        "/f", "`"$CertPath`"",
        "/p", "`"$CertPassword`"",
        "/fd", "SHA256",
        "/t", "http://timestamp.digicert.com",
        "/v",
        "`"$exePath`""
    )

    $signProcess = Start-Process -FilePath $signtoolPath -ArgumentList $signArgs -NoNewWindow -Wait -PassThru

    if ($signProcess.ExitCode -eq 0) {
        Write-Host ""
        Write-Host "✅ Ejecutable firmado exitosamente" -ForegroundColor Green
        Write-Host ""

        # Verificar firma
        Write-Host "🔍 Verificando firma digital..." -ForegroundColor Yellow
        $verifyArgs = @("verify", "/pa", "/v", "`"$exePath`"")
        & $signtoolPath $verifyArgs
    } else {
        Write-Host ""
        Write-Host "❌ Error al firmar el ejecutable (código: $($signProcess.ExitCode))" -ForegroundColor Red
        Write-Host "   Verifica que el certificado y la contraseña sean correctos" -ForegroundColor Yellow
    }
    Write-Host ""
}

# Verificar si Windows Defender está activo
Write-Host "🛡️  Verificando Windows Defender..." -ForegroundColor Yellow
try {
    $defenderStatus = Get-MpComputerStatus -ErrorAction SilentlyContinue
    if ($defenderStatus.RealTimeProtectionEnabled) {
        Write-Host "   ⚠️  Windows Defender está activo" -ForegroundColor Yellow
        Write-Host "   Si el ejecutable es bloqueado, agrega una excepción:" -ForegroundColor Gray
        Write-Host ""
        Write-Host "   Add-MpPreference -ExclusionPath `"$PWD\$exePath`"" -ForegroundColor Cyan
        Write-Host ""
    } else {
        Write-Host "   ℹ️  Windows Defender no está activo" -ForegroundColor Gray
    }
} catch {
    Write-Host "   ℹ️  No se pudo verificar estado de Windows Defender" -ForegroundColor Gray
}
Write-Host ""

# Recomendaciones finales
Write-Host "📝 Recomendaciones:" -ForegroundColor Cyan
Write-Host "   1. Prueba el ejecutable: .\$exePath" -ForegroundColor Gray
Write-Host "   2. Escanea en VirusTotal.com para verificar detecciones" -ForegroundColor Gray
Write-Host "   3. Si es bloqueado por Defender, agrega excepción (ver comando arriba)" -ForegroundColor Gray

if (-not $Sign) {
    Write-Host "   4. Para distribución profesional, considera firmar digitalmente:" -ForegroundColor Gray
    Write-Host "      .\build_release.ps1 -Sign -CertPath tu_cert.pfx -CertPassword tu_pass" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "✨ Build completado exitosamente" -ForegroundColor Green
Write-Host "📦 Ejecutable: $exePath" -ForegroundColor Green
Write-Host ""

# Preguntar si desea ejecutar el programa
$response = Read-Host "¿Deseas ejecutar el programa ahora? (S/N)"
if ($response -eq 'S' -or $response -eq 's') {
    Write-Host ""
    Write-Host "🚀 Iniciando win_opt..." -ForegroundColor Cyan
    & ".\$exePath"
}
