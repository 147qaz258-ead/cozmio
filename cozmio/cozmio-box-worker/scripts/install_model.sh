#!/bin/bash
#
# install_model.sh - Download and install llama.cpp and GGUF model for Cozmio Box Worker
#
# This script is designed for Raspberry Pi 4 (aarch64) but can also be used on x86_64 Linux.
#
# Usage:
#   ./install_model.sh                    # Interactive install
#   ./install_model.sh --model <model>    # Specify model (qwen2-0.5b, phi2-mini)
#   ./install_model.sh --skip-llama       # Skip llama.cpp install (assume already installed)
#
# Models will be downloaded to /opt/cozmio/models/
# llama.cpp will be installed to /opt/llama.cpp/
#

set -e

# Configuration
INSTALL_BASE="/opt/cozmio"
MODELS_DIR="${INSTALL_BASE}/models"
LLAMA_DIR="${INSTALL_BASE}/llama.cpp"
LLAMA_SERVER_PATH="${LLAMA_DIR}/llama-server"

# Model options (GGUF format, quantized)
declare -A MODELS
MODELS["qwen2-0.5b"]="Qwen2-0.5B-Instruct-Q4_K_M.gguf"
MODELS["qwen2-1.5b"]="Qwen2-1.5B-Instruct-Q4_K_M.gguf"
MODELS["phi2-mini"]="phi2-mini.Q4_K_M.gguf"

# Default model
SELECTED_MODEL="qwen2-0.5b"

# Parse arguments
SKIP_LLAMA=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --model)
            SELECTED_MODEL="$2"
            shift 2
            ;;
        --skip-llama)
            SKIP_LLAMA=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--model <model>] [--skip-llama] [--help]"
            echo ""
            echo "Models available:"
            for m in "${!MODELS[@]}"; do
                echo "  - $m"
            done
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check if running as root or pi user
check_user() {
    if [[ $EUID -eq 0 ]]; then
        echo "Running as root - OK"
    elif [[ $(whoami) == "pi" ]]; then
        echo "Running as pi - OK"
    else
        echo "Warning: Not running as root or pi user."
        echo "You may need to use sudo for installation."
    fi
}

# Create directories
create_dirs() {
    echo "Creating directories..."
    if [[ $EUID -eq 0 ]]; then
        mkdir -p "$MODELS_DIR"
        mkdir -p "$LLAMA_DIR"
        chown -R pi:pi "$INSTALL_BASE" 2>/dev/null || true
    else
        mkdir -p "$MODELS_DIR"
        mkdir -p "$LLAMA_DIR"
    fi
    echo "Directories created:"
    echo "  - $MODELS_DIR"
    echo "  - $LLAMA_DIR"
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    case $arch in
        aarch64|arm64)
            echo "aarch64"
            ;;
        x86_64|amd64)
            echo "x86_64"
            ;;
        *)
            echo "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Download llama.cpp binary
download_llama() {
    local arch=$(detect_arch)
    echo "Detected architecture: $arch"

    if [[ -x "$LLAMA_SERVER_PATH" ]]; then
        echo "llama-server already installed at $LLAMA_SERVER_PATH"
        return 0
    fi

    echo "Downloading llama.cpp..."

    local LLAMA_VERSION="v0.0.0"  # Use latest release
    local BASE_URL="https://github.com/ggerganov/llama.cpp/releases/download/${LLAMA_VERSION}"

    case $arch in
        aarch64)
            local BIN_FILE="llama-linux-arm64-cpu-instruct.bin"
            ;;
        x86_64)
            local BIN_FILE="llama-linux-x86_64-cpu-instruct.bin"
            ;;
    esac

    local DOWNLOAD_URL="${BASE_URL}/${BIN_FILE}"

    echo "Downloading from: $DOWNLOAD_URL"
    curl -L -o "$LLAMA_DIR/llama-server" "$DOWNLOAD_URL"
    chmod +x "$LLAMA_DIR/llama-server"

    echo "llama.cpp installed to $LLAMA_DIR/llama-server"
}

# Build llama.cpp from source (if prebuilt not available)
build_llama() {
    echo "Building llama.cpp from source..."

    if ! command -v g++ &> /dev/null; then
        echo "Installing build dependencies..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y build-essential cmake
        fi
    fi

    local LLAMA_REPO="/tmp/llama.cpp"
    if [[ ! -d "$LLAMA_REPO" ]]; then
        git clone https://github.com/ggerganov/llama.cpp.git "$LLAMA_REPO"
    fi

    cd "$LLAMA_REPO"
    mkdir -p build && cd build
    cmake .. -DLLAMA_SERVER=ON -DLLAMA_BUILD_EXAMPLES=ON
    cmake --build . --config Release

    cp server/llama-server "$LLAMA_DIR/"
    chmod +x "$LLAMA_DIR/llama-server"

    echo "llama.cpp built and installed to $LLAMA_DIR/llama-server"
}

# Download GGUF model
download_model() {
    local model_key=$1
    local model_file=${MODELS[$model_key]}

    if [[ -z "$model_file" ]]; then
        echo "Unknown model: $model_key"
        echo "Available models: ${!MODELS[@]}"
        exit 1
    fi

    local model_path="${MODELS_DIR}/${model_file}"

    if [[ -f "$model_path" ]]; then
        echo "Model already exists at $model_path"
        return 0
    fi

    echo "Downloading model: $model_key ($model_file)"
    echo "This may take several minutes (model is ~400MB)..."

    # HuggingFace GGUF model URLs (using TheBloke models)
    local HF_BASE="https://huggingface.co/TheBloke"
    local MODEL_URL="${HF_BASE}/${model_key}-GGUF"

    # Try to download from HuggingFace
    # Note: You may need to accept model license first on HF
    if command -v huggingface-cli &> /dev/null; then
        echo "Downloading via huggingface-cli..."
        huggingface-cli download "${model_key}-GGUF" "${model_file}" --local-dir "${MODELS_DIR}"
    else
        echo "Downloading directly..."
        # Direct HF download URL pattern
        local DOWNLOAD_URL="https://huggingface.co/TheBloke/${model_key}-GGUF/resolve/main/${model_file}"
        curl -L -o "$model_path" "$DOWNLOAD_URL"
    fi

    if [[ -f "$model_path" ]]; then
        echo "Model downloaded to $model_path"
    else
        echo "Failed to download model"
        echo "Please download manually from: https://huggingface.co/${model_key}-GGUF"
        exit 1
    fi
}

# Create symlink to current model
setup_current_model() {
    local model_key=$1
    local model_file=${MODELS[$model_key]}
    local model_path="${MODELS_DIR}/${model_file}"

    if [[ ! -f "$model_path" ]]; then
        echo "Model file not found: $model_path"
        return 1
    fi

    # Create 'current' symlink
    ln -sf "$model_path" "${MODELS_DIR}/current.gguf"
    echo "Created symlink: ${MODELS_DIR}/current.gguf -> $model_file"
}

# Create box-model.toml example
create_config_example() {
    local example_file="${INSTALL_BASE}/config/box-model.toml.example"

    if [[ $EUID -eq 0 ]]; then
        mkdir -p "${INSTALL_BASE}/config"
        chown -R pi:pi "${INSTALL_BASE}/config" 2>/dev/null || true
    else
        mkdir -p "${INSTALL_BASE}/config"
    fi

    cat > "$example_file" << 'EOF'
# Cozmio Box Model Configuration
# Copy this to /opt/cozmio/config/box-model.toml and adjust as needed

# Provider: "llama_cpp" or "mock"
provider = "llama_cpp"

# Path to GGUF model file (or "mock" to use mock provider)
model_path = "/opt/cozmio/models/current.gguf"

# llama.cpp server URL (for llama_cpp provider)
server_url = "http://localhost:8080"

# Model context size (tokens)
context_size = 2048

# Number of CPU threads (0 = auto)
threads = 4

# Inference timeout in seconds
timeout_secs = 120
EOF

    echo "Created config example at $example_file"
}

# Main installation
main() {
    echo "=========================================="
    echo "Cozmio Box Model Installer"
    echo "=========================================="
    echo ""
    echo "Model: $SELECTED_MODEL"
    echo "Install base: $INSTALL_BASE"
    echo ""

    check_user
    create_dirs

    if [[ "$SKIP_LLAMA" == "false" ]]; then
        # Try to download prebuilt, fall back to source build
        if ! download_llama; then
            echo "Prebuilt not available, building from source..."
            build_llama
        fi
    else
        echo "Skipping llama.cpp installation"
    fi

    download_model "$SELECTED_MODEL"
    setup_current_model "$SELECTED_MODEL"
    create_config_example

    echo ""
    echo "=========================================="
    echo "Installation complete!"
    echo "=========================================="
    echo ""
    echo "Next steps:"
    echo "1. Start llama-server: ${LLAMA_DIR}/llama-server \\"
    echo "      -m ${MODELS_DIR}/current.gguf \\"
    echo "      -c 2048 --host 0.0.0.0 --port 8080"
    echo ""
    echo "2. Copy config example:"
    echo "   cp ${INSTALL_BASE}/config/box-model.toml.example \\"
    echo "      ${INSTALL_BASE}/config/box-model.toml"
    echo ""
    echo "3. Start box-worker"
}

main
