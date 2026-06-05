#!/usr/bin/env bash
set -euo pipefail

llama_cpp_dir="${LLAMA_CPP_DIR:-$PWD/llama.cpp}"
server_bin="${LLAMA_SERVER:-$llama_cpp_dir/build/bin/llama-server}"
model="${MODEL_PATH:?MODEL_PATH is required}"
host="${SERVER_HOST:-0.0.0.0}"
port="${SERVER_PORT:-8080}"
threads="${THREADS:-8}"
ctx_size="${CTX_SIZE:-2048}"
batch_size="${BATCH_SIZE:-256}"
n_gpu_layers="${N_GPU_LAYERS:-0}"

extra_args=()
if [ -n "${LLAMA_EXTRA_ARGS:-}" ]; then
  read -r -a extra_args <<< "$LLAMA_EXTRA_ARGS"
fi

exec "$server_bin" \
  -m "$model" \
  --host "$host" \
  --port "$port" \
  --threads "$threads" \
  --ctx-size "$ctx_size" \
  --batch-size "$batch_size" \
  --n-gpu-layers "$n_gpu_layers" \
  "${extra_args[@]}"
