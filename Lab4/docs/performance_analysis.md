# 性能测试与系统分析文档

## 1. 实验对象

| 项目 | 实测值 |
| --- | --- |
| 主机 CPU | 待实测填写 |
| 主机内存 | 待实测填写 |
| 主机 GPU | 待实测填写，没有 GPU 写无 |
| 从机 CPU/内存/GPU | 待实测填写 |
| 操作系统 | 待实测填写 |
| llama.cpp commit | 待实测填写 |
| 模型名称 | 待实测填写 |
| GGUF 量化格式 | 待实测填写 |
| 模型文件大小 | 待实测填写 |

硬件和系统环境由 `scripts/collect_env.sh` 生成，原始记录放入 `results/raw/env_*.md`。

## 2. 性能指标列表

| 指标 | 定义 | 合理性 |
| --- | --- | --- |
| 模型加载时间 | 从启动进程到模型加载完成的耗时 | 反映模型文件 I/O、mmap、内存页缓存和量化模型大小的影响 |
| Prompt eval 吞吐 | 处理输入 prompt token 的 tokens/s | 输入越长越重要，受 batch、上下文长度和 CPU/GPU 后端影响 |
| Decode 吞吐 | 生成阶段 tokens/s | 直接决定模型持续输出速度，是交互式体验的核心指标 |
| 首 token 延迟 | 提交请求到第一个 token 返回的时间 | 影响用户体感响应速度，和加载、排队、prompt eval 相关 |
| 总延迟 | 提交请求到完整输出结束的时间 | 适合比较不同配置、单机/RPC/Ray 的端到端效果 |
| 内存/RSS 占用 | 进程最大常驻内存或显存占用 | 判断量化格式、上下文窗口和 `--no-mmap` 是否造成资源压力 |
| 输出长度 | 输出字符数或 token 数 | 归一化吞吐和延迟，避免不同回答长度造成误判 |
| 成功率 | 成功请求数 / 总请求数 | Ray 多机调度和失败重试时必须记录稳定性 |

本实验实际测量至少包含总延迟、decode 吞吐、内存占用或输出长度中的三项；RPC 与 Ray 部分额外记录吞吐量和失败请求数。

## 3. 单机部署记录

| 项目 | 记录 |
| --- | --- |
| 模型 | 待实测填写 |
| 量化格式 | 待实测填写 |
| 部署方式 | 本地编译 llama.cpp，CPU/CUDA/Metal 后端待填写 |
| 运行命令 | 见 `docs/commands.md` |
| 成功推理截图 | `results/screenshots/single_inference_success.png` |

## 4. 测试任务设计

### 4.1 性能测试任务

性能测试使用 `data/prompts_quality.jsonl` 中的短 prompt 作为稳定输入，并通过 `config/llama_sweep.example.json` 改变参数：

| 配置名 | 关键参数 | 目的 |
| --- | --- | --- |
| `baseline` | 默认线程、`--ctx-size 2048`、`--batch-size 256` | 单机基线 |
| `threads_half` | 较少线程 | 观察 CPU 并行度不足 |
| `threads_full` | 物理核心数附近 | 观察多线程收益 |
| `batch_large` | 较大 batch | 观察 prompt eval 吞吐与内存占用变化 |
| `no_mmap` | `--no-mmap` | 观察加载时间和 RSS 变化 |
| `gpu_offload` | `--n-gpu-layers` | 有 GPU 时观察 offload 收益 |

### 4.2 质量测试任务

质量评估使用 5 个 prompt，覆盖中文问答、摘要、代码解释、推理题和课程相关问题：

| Prompt ID | 类别 | 文件 |
| --- | --- | --- |
| `quality_cn_qa` | 中文问答 | `data/prompts_quality.jsonl` |
| `quality_summary` | 摘要 | `data/prompts_quality.jsonl` |
| `quality_code` | 代码解释 | `data/prompts_quality.jsonl` |
| `quality_reasoning` | 推理题 | `data/prompts_quality.jsonl` |
| `quality_osh` | 课程相关 | `data/prompts_quality.jsonl` |

人工评估维度：

| 维度 | 评分说明 |
| --- | --- |
| 相关性 | 是否正面回答 prompt |
| 正确性 | 事实、代码和推理是否正确 |
| 连贯性 | 中文表达是否自然、结构是否清楚 |
| 简洁性 | 是否避免无关扩写 |
| 稳定性 | 不同配置下是否出现明显退化或重复 |

## 5. 单机性能结果

运行命令：

```bash
python3 scripts/llama_cli_benchmark.py \
  --llama-bin "$LLAMA_CPP_DIR/build/bin/llama-cli" \
  --model "$MODEL_PATH" \
  --prompts data/prompts_quality.jsonl \
  --configs config/llama_sweep.example.json \
  --out-dir results/raw
```

结果表格待实测后填写：

| 配置 | 线程 | batch | ctx | GPU layers | 加载时间 ms | Prompt eval t/s | Decode t/s | 总延迟 s | 最大 RSS MB | 备注 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| baseline | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 |
| threads_half | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 |
| threads_full | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 |
| batch_large | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 |
| no_mmap | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 |
| gpu_offload | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 无 GPU 可删除 |

### 5.1 参数影响分析

实测后从以下角度分析：

| 参数 | 预期现象 | 系统原因 |
| --- | --- | --- |
| `--threads` | 线程数增加通常提高 CPU 吞吐，但超过物理核心后收益下降 | 调度开销、缓存竞争和内存带宽限制 |
| `--batch-size` | 较大 batch 可能提高 prompt eval 吞吐，但增加内存占用 | 更大的批处理提高矩阵计算利用率，同时占用更多中间缓冲 |
| `--ctx-size` | 上下文越大，KV cache 占用越高 | KV cache 与层数、隐藏维度和上下文 token 数相关 |
| `--n-gpu-layers` | GPU offload 可降低 decode 延迟 | Transformer 层计算转移到 GPU，但受显存和 PCIe/统一内存影响 |
| `--no-mmap` | 可能增加加载时间和 RSS，但减少 page fault 抖动 | 模型从文件映射改为直接读入内存 |

## 6. 输出质量结果

| Prompt ID | baseline 输出摘要 | 优化配置输出摘要 | 差异分析 |
| --- | --- | --- | --- |
| `quality_cn_qa` | 待填 | 待填 | 待填 |
| `quality_summary` | 待填 | 待填 | 待填 |
| `quality_code` | 待填 | 待填 | 待填 |
| `quality_reasoning` | 待填 | 待填 | 待填 |
| `quality_osh` | 待填 | 待填 | 待填 |

分析重点：

1. 线程、batch、ctx 等性能参数通常不改变模型权重，本身不应显著改变语义质量。
2. `--temp`、`--top-p`、`--repeat-penalty` 等采样参数会直接影响输出随机性、重复度和稳定性。
3. 如果配置导致内存压力、上下文截断或异常退出，输出质量会间接受到影响。

## 7. RPC 多机推理结果

拓扑：

| 节点 | IP | 角色 | 后端 | 命令 |
| --- | --- | --- | --- | --- |
| host | 待填 | 主机 `llama-cli` | 待填 | `docs/commands.md` |
| worker-a | 待填 | 从机 `rpc-server` | 待填 | `docs/commands.md` |

结果表：

| 模式 | Prompt | 总延迟 s | Decode t/s | 网络 | 备注 |
| --- | --- | --- | --- | --- | --- |
| 单机 | 待填 | 待填 | 待填 | 无 RPC | 待填 |
| RPC 1 从机 | 待填 | 待填 | 待填 | 待填 | 待填 |
| RPC 多从机 | 选做 | 待填 | 待填 | 待填 | 待填 |

RPC 分析要点：

1. RPC 不保证比单机快，特别是模型较小或网络较慢时，通信和同步开销可能超过计算收益。
2. 若从机性能弱于主机，计算划分可能导致主机等待从机，出现拖尾延迟。
3. 有线局域网通常比无线网络更稳定，首 token 延迟和总延迟更低。
4. RPC 的收益更可能出现在模型较大、单机内存不足或从机有更强 GPU 后端的场景。

## 8. Ray 批量推理结果

运行命令见 `docs/ray_task.md`。

| 模式 | Prompt 数 | 总耗时 s | 平均延迟 s | P95 延迟 s | 吞吐 req/s | 失败数 | 说明 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 串行 | 30 | 待填 | 待填 | 待填 | 待填 | 待填 | 单进程逐个请求 |
| Ray 轮询 | 30 | 待填 | 待填 | 待填 | 待填 | 待填 | 多节点并发 |
| Ray 延迟感知 | 选做 | 待填 | 待填 | 待填 | 待填 | 待填 | 按历史平均延迟分配 |

Ray 分析要点：

1. Ray 的价值是提高批量请求吞吐，而不是加速单个 prompt。
2. 如果每个请求很短，Ray 调度开销和 HTTP 往返会占比较高。
3. llama-server 常驻后避免重复加载模型，因此批量推理应复用服务进程。
4. 多机并行受最慢节点、网络延迟、每台 server 并发能力和 prompt 长度分布影响。
5. 异构机器上固定轮询可能造成慢节点堆积，延迟感知或按权重分配通常更合理。

## 9. 结论

待实测后填写。建议结论包含：

1. 最适合本机的 llama.cpp 参数组合。
2. 单机和 RPC 的性能差异及原因。
3. Ray 批量推理相对串行执行的吞吐变化。
4. 当前实验的限制，例如模型较小、网络为无线、没有 GPU、机器数量不足等。
