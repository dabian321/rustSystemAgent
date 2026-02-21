#!/usr/bin/env bash
# Rust System Agent — 一键安装脚本
# 用法: ./install.sh  或  bash install.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC} $1"; }
ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
err()   { echo -e "${RED}[ERROR]${NC} $1"; }

# 获取脚本所在目录（即项目根目录）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 检查是否在项目根目录（存在 Cargo.toml 且名为 rust-system-agent）
if [[ ! -f "Cargo.toml" ]]; then
  err "未在项目根目录找到 Cargo.toml，请在 rust-system-agent 目录下执行此脚本。"
  exit 1
fi

echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Rust System Agent — 一键安装${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# ---------- 1. 检查/安装 Rust ----------
check_rust() {
  if command -v rustc &>/dev/null && command -v cargo &>/dev/null; then
    local version
    version=$(rustc --version 2>/dev/null | head -1)
    info "已检测到 Rust: $version"
    return 0
  fi
  return 1
}

install_rust() {
  info "未检测到 Rust，正在通过 rustup 安装..."
  if command -v curl &>/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # 加载环境以便当前脚本能使用 cargo
    export PATH="$HOME/.cargo/bin:$PATH"
    ok "Rust 安装完成"
  else
    err "未找到 curl，请先安装 curl 或手动安装 Rust: https://rustup.rs"
    exit 1
  fi
}

if ! check_rust; then
  install_rust
fi

# 确保 cargo 在 PATH 中（若刚安装）
export PATH="${HOME:-/home/$(whoami)}/.cargo/bin:$PATH"
if ! command -v cargo &>/dev/null; then
  err "未找到 cargo，请将 ~/.cargo/bin 加入 PATH 后重试。"
  exit 1
fi

# ---------- 2. 编译 Release ----------
info "正在编译 (cargo build --release)..."
if ! cargo build --release; then
  err "编译失败，请检查错误信息。"
  exit 1
fi
ok "编译完成"

BINARY_SRC="$SCRIPT_DIR/target/release/rsa"
if [[ ! -f "$BINARY_SRC" ]]; then
  err "未找到二进制: $BINARY_SRC"
  exit 1
fi

# ---------- 3. 选择安装目录 ----------
INSTALL_DIR=""
if [[ -w "$HOME/.local/bin" ]]; then
  INSTALL_DIR="$HOME/.local/bin"
elif [[ -w "/usr/local/bin" ]]; then
  INSTALL_DIR="/usr/local/bin"
else
  # 尝试创建 ~/.local/bin
  mkdir -p "$HOME/.local/bin" 2>/dev/null && INSTALL_DIR="$HOME/.local/bin"
fi

if [[ -z "$INSTALL_DIR" ]]; then
  warn "无法写入 ~/.local/bin，将使用 sudo 安装到 /usr/local/bin"
  INSTALL_DIR="/usr/local/bin"
  sudo cp "$BINARY_SRC" "$INSTALL_DIR/rsa"
  sudo chmod +x "$INSTALL_DIR/rsa"
else
  mkdir -p "$INSTALL_DIR"
  cp "$BINARY_SRC" "$INSTALL_DIR/rsa"
  chmod +x "$INSTALL_DIR/rsa"
fi
ok "已安装到: $INSTALL_DIR/rsa"

# 提示 PATH
if [[ "$INSTALL_DIR" == "$HOME/.local/bin" ]]; then
  if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    warn "请将 ~/.local/bin 加入 PATH，例如在 ~/.bashrc 或 ~/.zshrc 中添加："
    echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
  fi
fi

# ---------- 4. 配置文件 ----------
ENV_FILE="$SCRIPT_DIR/.env"
if [[ ! -f "$ENV_FILE" ]]; then
  if [[ -f "$SCRIPT_DIR/.env.example" ]]; then
    cp "$SCRIPT_DIR/.env.example" "$ENV_FILE"
    info "已从 .env.example 创建 .env，请编辑并填入 API Key。"
  else
    info "请在该项目根目录创建 .env 并配置 API Key，例如："
    echo ""
    echo "  OPENROUTER_API_KEY=sk-or-v1-your-key"
    echo "  MODEL_NAME=google/gemini-2.5-flash"
    echo ""
  fi
else
  ok "已存在 .env 配置"
fi

# ---------- 5. 可选：交互模式别名 ----------
add_alias() {
  local rc_file="$1"
  local line="alias rasi='rsa interactive'"
  if [[ -f "$rc_file" ]] && ! grep -q "alias rasi=" "$rc_file" 2>/dev/null; then
    echo "" >> "$rc_file"
    echo "# Rust System Agent 交互模式" >> "$rc_file"
    echo "$line" >> "$rc_file"
    ok "已在 $rc_file 中添加: $line"
    return 0
  fi
  return 1
}

if [[ -f "$HOME/.bashrc" ]]; then
  add_alias "$HOME/.bashrc" || true
fi
if [[ -f "$HOME/.zshrc" ]]; then
  add_alias "$HOME/.zshrc" || true
fi

# ---------- 完成 ----------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  安装完成${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "  单次查询:  rsa '你的问题'"
echo "  交互模式:  rasi  或  rsa interactive"
echo ""
echo "  若刚添加了 alias，请执行: source ~/.bashrc  或  source ~/.zshrc"
echo ""
