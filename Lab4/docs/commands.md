# 命令记录

本文件用于记录实验执行过的命令。实测后请保留具体 IP、端口、模型路径、commit 和输出文件名。

## 1. 环境记录

```bash
cd Lab4
source config/experiment.env
./scripts/collect_env.sh results/raw/env_$(hostname).md
```

## 2. 编译 llama.cpp

CPU + RPC：

```bash
cd Lab4
BACKEND=cpu GGML_RPC=ON LLAMA_CPP_DIR="$PWD/llama.cpp" ./scripts/setup_llama_cpp.sh
```

CUDA + RPC：

```bash
cd Lab4
BACKEND=cuda GGML_RPC=ON LLAMA_CPP_DIR="$PWD/llama.cpp" ./scripts/setup_llama_cpp.sh
```

记录 commit：

```bash
git -C "$LLAMA_CPP_DIR" rev-parse HEAD
```

## 3. 单机推理

```bash
"$LLAMA_CPP_DIR/build/bin/llama-cli" \
  -m "$MODEL_PATH" \
  -p "请用三句话解释操作系统中的进程调度。" \
  -n 128 \
  --single-turn \
  --threads 8 \
  --ctx-size 2048 \
  --batch-size 256 \
  --temp 0.2
```

## 4. 参数扫描

```bash
python3 scripts/llama_cli_benchmark.py \
  --llama-bin "$LLAMA_CPP_DIR/build/bin/llama-cli" \
  --model "$MODEL_PATH" \
  --prompts data/prompts_quality.jsonl \
  --configs config/llama_sweep.example.json \
  --out-dir results/raw
```

## 5. 质量评估

```bash
./scripts/run_quality_prompts.sh
```

## 6. llama-bench

```bash
THREADS=8 BATCH_SIZE=256 N_PROMPT=512 N_GEN=128 REPETITIONS=3 N_GPU_LAYERS=0 ./scripts/run_llama_bench.sh
```

## 7. RPC 从机

在 worker 节点执行：

```bash
cd Lab4
source config/experiment.env
RPC_EXTRA_ARGS="-H 0.0.0.0 -t 4" RPC_PORT=50052 ./scripts/start_rpc_server.sh
```

## 8. RPC 主机

在 host 节点执行：

```bash
cd Lab4
source config/experiment.env
RPC_SERVERS="10.210.218.47:50052" \
PROMPT="请解释 llama.cpp RPC 后端为什么可能受网络延迟影响。" \
./scripts/run_rpc_inference.sh
```

本次实测网络为手机热点，VMware 从机桥接到热点网卡后获得地址 `10.210.218.47`。如果虚拟机地址是 `192.168.247.x`，说明仍在 VMware NAT 网段，热点中的另一台电脑不能直接访问该地址。

## 9. llama-server

每台推理节点执行：

```bash
cd Lab4
source config/experiment.env
SERVER_HOST=0.0.0.0 SERVER_PORT=8080 THREADS=8 CTX_SIZE=2048 BATCH_SIZE=256 \
  ./scripts/start_llama_server.sh
```

健康检查：

```bash
curl http://127.0.0.1:8080/health
```

## 10. Ray 集群

Head：

```bash
ray start --head --dashboard-host=0.0.0.0
```

本次最终结果中 Ray Task 运行在 Head A WSL 内，通过 HTTP 调用本机和从机的 `llama-server`，没有让 VMware 从机加入 Ray 集群。若需要多 Ray 节点，可执行：

```bash
ray start --address='192.168.1.10:6379'
```

状态：

```bash
ray status
```

## 11. Ray 批量推理

串行：

```bash
python3 scripts/ray_batch_infer.py \
  --mode serial \
  --config config/ray_servers.final.json \
  --prompts data/prompts_batch.jsonl \
  --out results/raw/ray_serial.jsonl
```

Ray 轮询：

```bash
python3 scripts/ray_batch_infer.py \
  --mode ray-round-robin \
  --ray-address auto \
  --config config/ray_servers.final.json \
  --prompts data/prompts_batch.jsonl \
  --out results/raw/ray_round_robin.jsonl
```

汇总：

```bash
python3 scripts/summarize_results.py results/raw/ray_*.jsonl \
  --out results/raw/ray_summary.md
```

## 12. 截图记录

| 截图 | 对应命令 |
| --- | --- |
| `quality_cn_qa_desktop_ck52vt6.png` | 第 3 或第 5 节，中文问答 |
| `quality_summary_desktop_ck52vt6.png` | 第 3 或第 5 节，摘要 |
| `quality_code_desktop_ck52vt6.png` | 第 3 或第 5 节，代码解释 |
| `quality_reasoning_desktop_ck52vt6.png` | 第 3 或第 5 节，推理题 |
| `quality_osh_desktop_ck52vt6.png` | 第 3 或第 5 节，课程相关问题 |
| `llama_benchmark_table.png` | 第 4 或 6 节，可后续补截图 |
| `rpc_worker_server_vm_c6h14.png` | 第 7 节 |
| `rpc_host_inference_desktop_ck52vt6.png` | 第 8 节 |
| `ray_status.png` | 第 10 节 |
| `ray_host.png` | 第 9 和第 11 节，主机 `llama-server` 请求处理日志 |
| `ray_workers.png` | 第 9 和第 11 节，从机 `llama-server` 请求处理日志 |
