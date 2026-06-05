# Ray 选择性必做任务说明

## 1. 任务选择

本组选择 Ray 方向，完成“多机批量推理任务调度”。Ray 作为调度层，将一批 prompt 分发到多个已经启动的 llama.cpp HTTP 推理服务上，并记录每个请求的开始时间、结束时间、总耗时和输出长度。

## 2. 系统结构

```text
              Ray head
                 |
      -------------------------
      |                       |
  Ray worker              Ray worker
      |                       |
 llama-server            llama-server
  node-a:8080             node-b:8080
```

每台机器运行一个 `llama-server`，Ray head 读取 `data/prompts_batch.jsonl`，使用 `scripts/ray_batch_infer.py` 将请求分配给不同 server。

资源不足时允许单机多进程模拟：

```text
              Ray local
                 |
      -------------------------
      |                       |
 llama-server :8080       llama-server :8081
```

如果使用模拟方案，报告中需要说明同一台机器上的多个 server 会争用 CPU、内存和磁盘 I/O，因此结果只能反映调度流程，不代表真实多机吞吐。

## 3. 节点配置记录

| 节点 | IP | Ray 角色 | llama-server 端口 | CPU/内存/GPU | 模型和量化 |
| --- | --- | --- | --- | --- | --- |
| node-a | 待填 | head | 8080 | 待填 | 待填 |
| node-b | 待填 | worker | 8080 | 待填 | 待填 |

## 4. Prompt 数据集

批量任务使用 `data/prompts_batch.jsonl`，共 30 条 prompt，覆盖：

| 类别 | 数量 | 目的 |
| --- | --- | --- |
| 课程知识问答 | 8 | 测试中文技术问答 |
| 代码解释 | 6 | 测试结构化解释能力 |
| 摘要任务 | 6 | 测试较长输入 |
| 推理题 | 5 | 测试逻辑链路 |
| 系统分析 | 5 | 贴合 OSH 实验主题 |

Ray 必做只要求不少于 20 条，30 条可以同时支撑负载均衡加分项。

## 5. llama-server 启动

每台机器执行：

```bash
cd Lab4
source config/experiment.env
SERVER_HOST=0.0.0.0 SERVER_PORT=8080 THREADS=8 CTX_SIZE=2048 BATCH_SIZE=256 \
  ./scripts/start_llama_server.sh
```

确认服务可访问：

```bash
curl http://127.0.0.1:8080/health
curl http://<node-ip>:8080/completion \
  -H 'Content-Type: application/json' \
  -d '{"prompt":"测试 Ray 调度。","n_predict":32,"temperature":0.2}'
```

## 6. Ray 集群启动

Head 节点：

```bash
ray start --head --node-ip-address=<head-ip> --port=6379 --dashboard-host=0.0.0.0
```

Worker 节点：

```bash
ray start --address='<head-ip>:6379'
```

状态检查：

```bash
ray status
```

截图保存到 `results/screenshots/ray_status.png`。

## 7. 配置文件

复制并编辑：

```bash
cp config/ray_servers.example.json config/ray_servers.json
```

将 `servers` 改为实际机器：

```json
{
  "servers": [
    {"name": "node-a", "url": "http://192.168.1.10:8080", "weight": 1},
    {"name": "node-b", "url": "http://192.168.1.11:8080", "weight": 1}
  ]
}
```

## 8. 执行方式对比

### 8.1 串行执行

```bash
python3 scripts/ray_batch_infer.py \
  --mode serial \
  --config config/ray_servers.json \
  --prompts data/prompts_batch.jsonl \
  --out results/raw/ray_serial.jsonl
```

串行模式不使用 Ray，作为端到端基线。

### 8.2 Ray 轮询分配

```bash
python3 scripts/ray_batch_infer.py \
  --mode ray-round-robin \
  --ray-address auto \
  --config config/ray_servers.json \
  --prompts data/prompts_batch.jsonl \
  --out results/raw/ray_round_robin.jsonl
```

轮询模式按 prompt 顺序将请求分配给 server，适合硬件配置相近的节点。

### 8.3 Ray 按权重分配

```bash
python3 scripts/ray_batch_infer.py \
  --mode ray-weighted \
  --ray-address auto \
  --config config/ray_servers.json \
  --prompts data/prompts_batch.jsonl \
  --out results/raw/ray_weighted.jsonl
```

在 `config/ray_servers.json` 中为更快的节点设置更高 `weight`。

### 8.4 Ray 延迟感知分配

```bash
python3 scripts/ray_batch_infer.py \
  --mode ray-latency-aware \
  --ray-address auto \
  --concurrency 4 \
  --config config/ray_servers.json \
  --prompts data/prompts_batch.jsonl \
  --out results/raw/ray_latency_aware.jsonl
```

延迟感知模式根据已完成请求的平均延迟和当前 in-flight 数选择节点，可作为负载均衡加分项材料。

## 9. 结果汇总

```bash
python3 scripts/summarize_results.py results/raw/ray_*.jsonl \
  --out results/raw/ray_summary.md
```

报告中填写：

| 模式 | 总耗时 s | 平均延迟 s | P95 延迟 s | 吞吐 req/s | 失败数 |
| --- | --- | --- | --- | --- | --- |
| serial | 待填 | 待填 | 待填 | 待填 | 待填 |
| ray-round-robin | 待填 | 待填 | 待填 | 待填 | 待填 |
| ray-weighted | 选做 | 待填 | 待填 | 待填 | 待填 |
| ray-latency-aware | 选做 | 待填 | 待填 | 待填 | 待填 |

## 10. 分析要点

1. 串行执行的总耗时近似等于每个请求延迟之和。
2. Ray 并行执行的总耗时取决于最慢的一批请求，而非所有请求简单相加。
3. prompt 长短不均会导致负载不均，轮询并不总是最优。
4. 如果 llama-server 自身只能有效处理一个请求，节点内并发过高会造成排队。
5. 多机环境中 HTTP 往返、Ray object store 序列化、节点时钟差异和网络拥塞都可能影响结果。

## 11. 必做项核对

| 要求 | 完成位置 |
| --- | --- |
| Ray 单机或多机部署说明 | 本文第 6 节 |
| 至少 2 台机器运行 llama.cpp 服务或模拟方案 | 本文第 2、5 节 |
| 不少于 20 个 prompt | `data/prompts_batch.jsonl` |
| 使用 Ray Task 或 Actor 分发请求 | `scripts/ray_batch_infer.py` |
| 收集开始/结束时间、总耗时、输出长度 | `results/raw/ray_*.jsonl` |
| 比较至少两种执行方式 | 本文第 8、9 节 |
| 分析调度开销和系统原因 | `docs/performance_analysis.md` |
