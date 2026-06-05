#!/usr/bin/env bash
set -euo pipefail

out="${1:-results/raw/env_$(hostname)_$(date +%Y%m%d_%H%M%S).md}"
mkdir -p "$(dirname "$out")"

{
  echo "# 环境记录"
  echo
  echo "- time: $(date -Is)"
  echo "- host: $(hostname)"
  echo "- pwd: $(pwd)"
  echo
  echo "## OS"
  uname -a || true
  if [ -r /etc/os-release ]; then
    cat /etc/os-release
  fi
  echo
  echo "## CPU"
  lscpu || true
  echo
  echo "## Memory"
  free -h || true
  echo
  echo "## Disk"
  df -h . || true
  echo
  echo "## Network"
  ip addr show || ifconfig || true
  echo
  echo "## GPU"
  nvidia-smi || true
  rocminfo || true
  echo
  echo "## Python"
  python3 --version || true
  echo
  echo "## Ray"
  ray --version || true
  echo
  echo "## llama.cpp"
  if [ -n "${LLAMA_CPP_DIR:-}" ] && [ -d "$LLAMA_CPP_DIR/.git" ]; then
    git -C "$LLAMA_CPP_DIR" rev-parse HEAD || true
    git -C "$LLAMA_CPP_DIR" status --short || true
  else
    echo "LLAMA_CPP_DIR is not set or is not a git checkout."
  fi
} > "$out"

echo "Wrote $out"
