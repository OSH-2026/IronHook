#!/usr/bin/env bash
set -euo pipefail

llama_cpp_dir="${LLAMA_CPP_DIR:-$PWD/llama.cpp}"
bench_bin="${LLAMA_BENCH:-$llama_cpp_dir/build/bin/llama-bench}"
model="${MODEL_PATH:?MODEL_PATH is required}"
threads="${THREADS:-8}"
ctx_size="${CTX_SIZE:-2048}"
batch_size="${BATCH_SIZE:-256}"
n_gpu_layers="${N_GPU_LAYERS:-0}"
out="${OUT:-results/raw/llama_bench_$(date +%Y%m%d_%H%M%S).txt}"

mkdir -p "$(dirname "$out")"

"$bench_bin" \
  -m "$model" \
  -t "$threads" \
  -c "$ctx_size" \
  -b "$batch_size" \
  -ngl "$n_gpu_layers" 2>&1 | tee "$out"

echo "Wrote $out"
