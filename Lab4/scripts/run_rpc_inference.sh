#!/usr/bin/env bash
set -euo pipefail

llama_cpp_dir="${LLAMA_CPP_DIR:-$PWD/llama.cpp}"
llama_cli="${LLAMA_CLI:-$llama_cpp_dir/build/bin/llama-cli}"
model="${MODEL_PATH:?MODEL_PATH is required}"
rpc_servers="${RPC_SERVERS:?RPC_SERVERS is required, for example 192.168.1.11:50052}"
prompt="${PROMPT:-请解释 llama.cpp RPC 推理的主要开销。}"
n_predict="${N_PREDICT:-128}"
threads="${THREADS:-8}"
ctx_size="${CTX_SIZE:-2048}"
batch_size="${BATCH_SIZE:-256}"
temperature="${TEMPERATURE:-0.2}"

extra_args=()
if [ -n "${LLAMA_EXTRA_ARGS:-}" ]; then
  read -r -a extra_args <<< "$LLAMA_EXTRA_ARGS"
fi

exec "$llama_cli" \
  -m "$model" \
  -p "$prompt" \
  -n "$n_predict" \
  --single-turn \
  --threads "$threads" \
  --ctx-size "$ctx_size" \
  --batch-size "$batch_size" \
  --temp "$temperature" \
  --rpc "$rpc_servers" \
  "${extra_args[@]}"
