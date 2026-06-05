#!/usr/bin/env bash
set -euo pipefail

llama_cpp_dir="${LLAMA_CPP_DIR:-$PWD/llama.cpp}"
llama_cli="${LLAMA_CLI:-$llama_cpp_dir/build/bin/llama-cli}"
model="${MODEL_PATH:?MODEL_PATH is required}"

python3 scripts/llama_cli_benchmark.py \
  --llama-bin "$llama_cli" \
  --model "$model" \
  --prompts data/prompts_quality.jsonl \
  --configs config/quality_configs.example.json \
  --out-dir results/raw
