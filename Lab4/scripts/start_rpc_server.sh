#!/usr/bin/env bash
set -euo pipefail

llama_cpp_dir="${LLAMA_CPP_DIR:-$PWD/llama.cpp}"
rpc_bin="${RPC_SERVER:-$llama_cpp_dir/build/bin/rpc-server}"
port="${RPC_PORT:-50052}"

extra_args=()
if [ -n "${RPC_EXTRA_ARGS:-}" ]; then
  read -r -a extra_args <<< "$RPC_EXTRA_ARGS"
fi

exec "$rpc_bin" -p "$port" "${extra_args[@]}"
