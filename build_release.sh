#!/bin/bash

# Script de compilación optimizada para win_opt (desde Linux)
# Reduce detecciones de falsos positivos en antivirus

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔨 win_opt - Script de compilación optimizada${NC}"
echo -e "${BLUE}==============================================${NC}"
echo ""

# Función para mostrar uso
show_usage() {
    echo "Uso: $0 [opciones]"
    echo ""
    echo "Opciones:"
    echo "  --clean          Limpiar builds anteriores"
    echo "  --install-deps   Instalar dependencias necesarias"
    echo "  --help           Mostrar esta ayuda"
    echo ""
}

# Parsear argumentos
CLEAN=false
INSTALL_DEPS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --install-deps)
            INSTALL_DEPS=true
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Argumento desconocido: $1${NC}"
            show_usage
            exit 1
            ;;
    esac
done

# Instalar dependencias si se solicita
if [ "$INSTALL_DEPS" = true ]; then
    echo -e "${YELLOW}📦 Instalando dependencias...${NC}"

    # Verificar si cross está instalado
    if ! command -v cross &> /dev/null; then
        echo -e "${YELLOW}   Instalando 'cross' para cross-compilation...${NC}"
        cargo install cross --git https://github.com/cross-rs/cross
    else
        echo -e "${GREEN}   ✅ 'cross' ya está instalado${NC}"
    fi

    # Agregar target de Windows
    echo -e "${YELLOW}   Agregando target x86_64-pc-windows-gnu...${NC}"
    rustup target add x86_64-pc-windows-gnu

    echo -e "${GREEN}✅ Dependencias instaladas${NC}"
    echo ""
fi

# Limpiar builds anteriores si se solicita
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}🧹 Limpiando builds anteriores...${NC}"
    cargo clean
    echo -e "${GREEN}✅ Limpieza completada${NC}"
    echo ""
fi

# Verificar que Rust esté instalado
echo -e "${YELLOW}🔍 Verificando instalación de Rust...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Error: Rust no está instalado${NC}"
    echo -e "${RED}Instala Rust desde: https://rustup.rs/${NC}"
    exit 1
fi

RUST_VERSION=$(cargo --version)
echo -e "${GREEN}✅ $RUST_VERSION${NC}"
echo ""

# Verificar si cross está disponible
USE_CROSS=false
if command -v cross &> /dev/null; then
    echo -e "${GREEN}✅ Usando 'cross' para cross-compilation${NC}"
    USE_CROSS=true
    BUILD_CMD="cross"
else
    echo -e "${YELLOW}⚠️  'cross' no encontrado, usando cargo nativo${NC}"
    echo -e "${GRAY}   Para mejor compatibilidad, instala cross con:${NC}"
    echo -e "${GRAY}   cargo install cross --git https://github.com/cross-rs/cross${NC}"
    BUILD_CMD="cargo"

    # Verificar que el target esté instalado
    echo -e "${YELLOW}🔍 Verificando target x86_64-pc-windows-gnu...${NC}"
    if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
        echo -e "${YELLOW}📦 Instalando target x86_64-pc-windows-gnu...${NC}"
        rustup target add x86_64-pc-windows-gnu
    fi
fi
echo ""

# Compilar con optimizaciones
echo -e "${BLUE}🚀 Compilando en modo release con optimizaciones...${NC}"
echo -e "${GRAY}   - opt-level: z (tamaño)${NC}"
echo -e "${GRAY}   - lto: true${NC}"
echo -e "${GRAY}   - strip: true${NC}"
echo -e "${GRAY}   - codegen-units: 1${NC}"
echo -e "${GRAY}   - panic: abort${NC}"
echo ""

# Ejecutar build
$BUILD_CMD build --release --target x86_64-pc-windows-gnu

echo ""
echo -e "${GREEN}✅ Compilación exitosa${NC}"
echo ""

# Ruta del ejecutable
EXE_PATH="target/x86_64-pc-windows-gnu/release/win_opt.exe"

# Mostrar información del binario
echo -e "${BLUE}📊 Información del binario:${NC}"
echo -e "${GRAY}   Ruta: $EXE_PATH${NC}"

if [ -f "$EXE_PATH" ]; then
    FILE_SIZE=$(du -h "$EXE_PATH" | cut -f1)
    echo -e "${GRAY}   Tamaño: $FILE_SIZE${NC}"

    # Calcular hash SHA256
    if command -v sha256sum &> /dev/null; then
        HASH=$(sha256sum "$EXE_PATH" | cut -d' ' -f1)
        echo -e "${GRAY}   SHA256: $HASH${NC}"
    fi

    # Verificar que es un ejecutable PE válido
    if command -v file &> /dev/null; then
        FILE_TYPE=$(file "$EXE_PATH")
        if echo "$FILE_TYPE" | grep -q "PE32+"; then
            echo -e "${GREEN}   ✅ Ejecutable PE de Windows válido (64-bit)${NC}"
        else
            echo -e "${YELLOW}   ⚠️  Advertencia: Tipo de archivo inesperado${NC}"
            echo -e "${GRAY}   $FILE_TYPE${NC}"
        fi
    fi
else
    echo -e "${RED}❌ Error: El ejecutable no se generó en la ruta esperada${NC}"
    exit 1
fi
echo ""

# Recomendaciones finales
echo -e "${BLUE}📝 Recomendaciones:${NC}"
echo -e "${GRAY}   1. Prueba el ejecutable en Windows${NC}"
echo -e "${GRAY}   2. Escanea el binario en VirusTotal.com${NC}"
echo -e "${GRAY}   3. Si Windows Defender lo bloquea, agrega una excepción en Windows:${NC}"
echo -e "${GRAY}      Add-MpPreference -ExclusionPath \"C:\\ruta\\a\\win_opt.exe\"${NC}"
echo -e "${GRAY}   4. Para distribución, considera firmar digitalmente el ejecutable${NC}"
echo ""

echo -e "${GREEN}✨ Build completado exitosamente${NC}"
echo -e "${GREEN}📦 Ejecutable: $EXE_PATH${NC}"
echo ""
echo -e "${GRAY}💡 Tip: Copia el ejecutable a Windows para probarlo:${NC}"
echo -e "${GRAY}   scp $EXE_PATH usuario@windows-pc:C:\\Users\\usuario\\Desktop\\${NC}"
