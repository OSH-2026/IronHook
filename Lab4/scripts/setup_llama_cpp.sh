#!/usr/bin/env bash
set -euo pipefail

repo="${LLAMA_CPP_DIR:-$PWD/llama.cpp}"
backend="${BACKEND:-cpu}"
build_dir="${BUILD_DIR:-build}"
ggml_rpc="${GGML_RPC:-ON}"
jobs="${JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)}"

if [ ! -d "$repo/.git" ]; then
  git clone https://github.com/ggml-org/llama.cpp "$repo"
fi

cmake_args=("-B" "$repo/$build_dir" "-S" "$repo" "-DGGML_RPC=$ggml_rpc")

case "$backend" in
  cpu)
    ;;
  cuda)
    cmake_args+=("-DGGML_CUDA=ON")
    ;;
  metal)
    cmake_args+=("-DGGML_METAL=ON")
    ;;
  hip)
    cmake_args+=("-DGGML_HIP=ON")
    ;;
  vulkan)
    cmake_args+=("-DGGML_VULKAN=ON")
    ;;
  *)
    echo "Unknown BACKEND: $backend" >&2
    exit 2
    ;;
esac

cmake "${cmake_args[@]}"
cmake --build "$repo/$build_dir" --config Release -j "$jobs"

echo "Built llama.cpp in $repo/$build_dir"
