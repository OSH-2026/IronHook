# Lab4 部署说明文档

## 1. 实验目标

本实验部署 llama.cpp 的单机推理、RPC 多机分布式推理，以及 Ray 批量推理任务调度。实验重点是观察线程、内存、I/O、量化格式、RPC 通信和任务调度对推理系统的影响，不以最高性能为目标。

## 2. 推荐环境

| 项目 | 推荐配置 | 记录位置 |
| --- | --- | --- |
| 操作系统 | Linux x86_64，Ubuntu 22.04/24.04 或同类发行版 | `results/raw/env_*.md` |
| 编译工具 | `git`、`cmake`、`gcc/g++` 或 `clang/clang++` | `docs/commands.md` |
| Python | Python 3.10+ | `requirements.txt` |
| 模型 | 1B/3B 级 GGUF 量化模型，推荐 Q4_K_M 或 Q5_K_M | 报告表格 |
| 网络 | 同一局域网，两台机器互相可访问 | RPC/Ray 章节 |

示例模型可以选择 Qwen2.5-1.5B-Instruct 的 GGUF Q4_K_M 量化版本。若小组机器内存充足，可改用 3B 或 7B 级模型，但需要在报告中说明模型名称、参数规模、量化格式和文件大小。

## 3. 文件准备

模型文件不提交到仓库，建议本地路径如下：

```bash
mkdir -p Lab4/models
# 将 GGUF 文件放入 Lab4/models/，例如：
# Lab4/models/qwen2.5-1.5b-instruct-q4_k_m.gguf
```

复制配置模板：

```bash
cp Lab4/config/experiment.env.example Lab4/config/experiment.env
cp Lab4/config/ray_servers.example.json Lab4/config/ray_servers.json
```

编辑 `Lab4/config/experiment.env`，至少填写：

```bash
export LLAMA_CPP_DIR="$PWD/llama.cpp"
export MODEL_PATH="/absolute/path/to/model.gguf"
export MODEL_NAME="Qwen2.5-1.5B-Instruct"
export QUANTIZATION="Q4_K_M"
```

## 4. llama.cpp 编译

CPU 后端并启用 RPC：

```bash
cd Lab4
BACKEND=cpu GGML_RPC=ON LLAMA_CPP_DIR="$PWD/llama.cpp" ./scripts/setup_llama_cpp.sh
```

NVIDIA CUDA 后端并启用 RPC：

```bash
cd Lab4
BACKEND=cuda GGML_RPC=ON LLAMA_CPP_DIR="$PWD/llama.cpp" ./scripts/setup_llama_cpp.sh
```

Apple Metal 后端并启用 RPC：

```bash
cd Lab4
BACKEND=metal GGML_RPC=ON LLAMA_CPP_DIR="$PWD/llama.cpp" ./scripts/setup_llama_cpp.sh
```

编译完成后应能看到：

```bash
$LLAMA_CPP_DIR/build/bin/llama-cli
$LLAMA_CPP_DIR/build/bin/llama-server
$LLAMA_CPP_DIR/build/bin/rpc-server
```

## 5. 单机推理

记录硬件和系统环境：

```bash
cd Lab4
source config/experiment.env
./scripts/collect_env.sh results/raw/env_$(hostname).md
```

一次性推理：

```bash
"$LLAMA_CPP_DIR/build/bin/llama-cli" \
  -m "$MODEL_PATH" \
  -p "请用三句话解释操作系统中的进程调度。" \
  -n 128 \
  --threads 8 \
  --ctx-size 2048 \
  --batch-size 256 \
  --temp 0.2
```

批量质量评估：

```bash
cd Lab4
source config/experiment.env
./scripts/run_quality_prompts.sh
```

参数扫描：

```bash
cd Lab4
source config/experiment.env
python3 scripts/llama_cli_benchmark.py \
  --llama-bin "$LLAMA_CPP_DIR/build/bin/llama-cli" \
  --model "$MODEL_PATH" \
  --prompts data/prompts_quality.jsonl \
  --configs config/llama_sweep.example.json \
  --out-dir results/raw
```

## 6. llama.cpp RPC 多机推理

### 6.1 网络拓扑

建议至少两台机器：

| 角色 | 示例 IP | 任务 |
| --- | --- | --- |
| 主机 host | `192.168.1.10` | 运行 `llama-cli`，加载模型，连接 RPC 后端 |
| 从机 worker-a | `192.168.1.11` | 运行 `rpc-server`，提供计算后端 |

两台机器需要处于同一局域网，并开放 RPC 端口，例如 `50052`。记录网络命令：

```bash
ip addr
ping 192.168.1.11
nc -vz 192.168.1.11 50052
```

### 6.2 从机启动 rpc-server

在每台从机上执行：

```bash
cd Lab4
source config/experiment.env
RPC_PORT=50052 ./scripts/start_rpc_server.sh
```

### 6.3 主机连接 RPC 后端推理

在主机上执行：

```bash
cd Lab4
source config/experiment.env
RPC_SERVERS="192.168.1.11:50052" \
PROMPT="请解释 llama.cpp RPC 后端为什么可能受网络延迟影响。" \
./scripts/run_rpc_inference.sh
```

多台从机时使用逗号分隔：

```bash
RPC_SERVERS="192.168.1.11:50052,192.168.1.12:50052" ./scripts/run_rpc_inference.sh
```

## 7. llama-server 部署

Ray 任务调度需要每台推理节点上有一个 HTTP 服务：

```bash
cd Lab4
source config/experiment.env
SERVER_PORT=8080 THREADS=8 CTX_SIZE=2048 BATCH_SIZE=256 ./scripts/start_llama_server.sh
```

健康检查：

```bash
curl http://127.0.0.1:8080/health
```

一次 HTTP 推理测试：

```bash
curl http://127.0.0.1:8080/completion \
  -H 'Content-Type: application/json' \
  -d '{"prompt":"用一句话解释 Ray 的 Task。","n_predict":64,"temperature":0.2}'
```

## 8. Ray 部署

安装依赖：

```bash
cd Lab4
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

Head 节点：

```bash
ray start --head --node-ip-address=192.168.1.10 --port=6379 --dashboard-host=0.0.0.0
```

Worker 节点：

```bash
ray start --address='192.168.1.10:6379'
```

查看集群状态：

```bash
ray status
```

运行串行基线：

```bash
python3 scripts/ray_batch_infer.py \
  --mode serial \
  --config config/ray_servers.json \
  --prompts data/prompts_batch.jsonl \
  --out results/raw/ray_serial.jsonl
```

运行 Ray 轮询调度：

```bash
python3 scripts/ray_batch_infer.py \
  --mode ray-round-robin \
  --ray-address auto \
  --config config/ray_servers.json \
  --prompts data/prompts_batch.jsonl \
  --out results/raw/ray_round_robin.jsonl
```

汇总结果：

```bash
python3 scripts/summarize_results.py results/raw/ray_*.jsonl \
  --out results/raw/ray_summary.md
```

## 9. 截图要求

建议至少保存以下截图到 `results/screenshots/`：

| 文件名建议 | 内容 |
| --- | --- |
| `single_inference_success.png` | 单机 `llama-cli` 成功推理终端输出 |
| `llama_benchmark_table.png` | 参数扫描或 `llama-bench` 结果 |
| `rpc_worker_server.png` | 从机 `rpc-server` 启动并接收连接 |
| `rpc_inference_success.png` | 主机 RPC 推理成功输出 |
| `ray_status.png` | `ray status` 或 Ray Dashboard |
| `ray_batch_result.png` | Ray 批量推理结果汇总 |

截图需要能看出机器名、命令或结果文件名，方便助教复现。
