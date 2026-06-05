# 性能测试与系统分析文档

## 1. 实验对象

| 项目 | 实测值 |
| --- | --- |
| 主机 CPU | 13th Gen Intel(R) Core(TM) i9-13900H，20 逻辑 CPU，10 核 / 20 线程 |
| 主机内存 | 15 GiB |
| 主机 GPU | 本次单机测试未使用 GPU；`nvidia-smi`、`rocminfo` 未检测到可用 GPU |
| 从机 CPU/内存/GPU | VMware Ubuntu 虚拟机 `c6h14-VMware-Virtual-Platform`；Intel(R) Core(TM) Ultra 9 185H，4 vCPU，内存 7.7 GiB，未检测到 CUDA |
| 操作系统 | 主机为 Ubuntu 24.04.4 LTS，WSL2，Linux 6.18.26.1-microsoft-standard-WSL2；从机为 VMware Ubuntu 虚拟机 |
| RPC/Ray 网络 | 手机热点局域网，VMware 桥接到热点网卡；从机地址 `10.210.218.47`，RPC 端口 `50052`，HTTP 端口 `8080` |
| llama.cpp commit | `2016bf2b3bca10e49e06a00586a8a2fde9f6cc32`，build `b9528-2016bf2b3` |
| 模型名称 | Qwen2.5-1.5B-Instruct GGUF |
| GGUF 量化格式 | Q4_K_M |
| 模型文件大小 | 1.1G，本地文件 `models/qwen2.5-1.5b-instruct-q4_k_m.gguf` |

硬件和系统环境由 `scripts/collect_env.sh` 生成，原始记录放入 `results/raw/env_*.md`。从机环境记录 `results/raw/env_c6h14.md` 中的网络地址是桥接调试前的 VMware NAT 快照，最终 RPC/Ray 运行地址以 `10.210.218.47` 为准。

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
| 模型 | Qwen2.5-1.5B-Instruct GGUF |
| 量化格式 | Q4_K_M |
| 部署方式 | 本地编译 llama.cpp，CPU 后端，编译时启用 `GGML_RPC=ON` |
| 运行命令 | 见 `docs/commands.md` |
| 成功推理截图 | `results/screenshots/quality_osh_desktop_ck52vt6.png` 等质量评估截图 |

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

原始数据来自 `results/raw/llama_cli_benchmark_20260605_230453.jsonl`。每个配置运行 5 个 prompt，记录端到端耗时、成功率和 `/usr/bin/time -v` 给出的最大 RSS。

| 配置 | 线程 | batch | ctx | GPU layers | 平均总延迟 s | 最短/最长 s | 平均最大 RSS MB | 最大 RSS MB | 成功率 | 备注 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `baseline` | 8 | 256 | 2048 | 0 | 5.31 | 2.77/7.63 | 1789.8 | 1793.3 | 5/5 | 基线配置 |
| `threads_half` | 4 | 256 | 2048 | 0 | 6.16 | 3.64/8.03 | 1790.6 | 1794.3 | 5/5 | 线程数减半，延迟上升 |
| `threads_full` | 12 | 256 | 2048 | 0 | 18.89 | 10.84/26.07 | 1789.5 | 1792.9 | 5/5 | WSL2 中显著变慢 |
| `batch_large` | 8 | 512 | 2048 | 0 | 5.69 | 3.40/7.25 | 1789.8 | 1792.6 | 5/5 | batch 从 256 增至 512 |
| `no_mmap` | 8 | 256 | 2048 | 0 | 5.40 | 3.04/7.41 | 1194.3 | 1197.7 | 5/5 | 禁用 mmap 后 RSS 明显下降 |
| `gpu_offload` | 8 | 256 | 2048 | 20 | 5.46 | 2.67/7.30 | 1789.7 | 1793.0 | 5/5 | 本机无可用 GPU，实际仍主要为 CPU 路径 |

`llama-bench` 使用新版参数重跑后得到 `results/raw/llama_bench_20260605_231015_fixed.txt`：

| 测试项 | 后端 | 线程 | batch | 吞吐 |
| --- | --- | --- | --- | --- |
| Prompt eval `pp512` | CPU | 8 | 256 | `199.57 ± 6.20 t/s` |
| Decode `tg128` | CPU | 8 | 256 | `38.42 ± 3.65 t/s` |

说明：曾有一次旧参数导致的无效 `llama-bench` 记录，原因是新版 `llama-bench` 不再接受旧脚本里的 `-c/--ctx-size` 参数；无效记录已清理，脚本已改为 `-p/--n-prompt` 与 `-n/--n-gen`。

### 5.1 参数影响分析

根据实测结果：

| 参数 | 实测现象 | 系统原因 |
| --- | --- | --- |
| `--threads` | 4 线程平均 6.16s，8 线程平均 5.31s，12 线程平均 18.89s | WSL2 环境下线程数超过有效并行范围后出现调度、缓存和异构核心竞争，CPU 利用率反而下降 |
| `--batch-size` | batch 512 平均 5.69s，略慢于 baseline | 该批 prompt 较短，batch 扩大未明显提高吞吐，反而带来额外缓冲和调度开销 |
| `--ctx-size` | 本次固定为 2048 | 该值足够覆盖测试 prompt；继续增大将提高 KV cache 内存压力，单机阶段未作为主变量扫描 |
| `--n-gpu-layers` | 设置 20 后平均 5.46s，与 baseline 接近 | 本机没有可用 GPU 后端，无法形成真正 offload；报告中不把该项作为 GPU 加速结论 |
| `--no-mmap` | 平均耗时 5.40s，与 baseline 接近；最大 RSS 从约 1.79GB 降到约 1.20GB | 在当前 WSL2 文件系统和页缓存状态下，禁用 mmap 改变了内存记账和映射方式；因模型较小，端到端延迟差异不大 |

综合来看，本机单机 CPU 推理的较优配置是 `--threads 8 --batch-size 256 --ctx-size 2048 --n-gpu-layers 0`。该配置平均延迟最低，内存占用稳定，且 `llama-bench` 的 decode 吞吐约为 38.42 t/s。

## 6. 输出质量结果

| Prompt ID | 截图 | baseline 输出摘要 | 差异分析 |
| --- | --- | --- | --- |
| `quality_cn_qa` | `results/screenshots/quality_cn_qa_desktop_ck52vt6.png` | 围绕“读万卷书不如行万里路”给出实践经验、旅行体验、技能培养和个人成长等例子 | 输出相关、结构清晰，但有一定套话和重复 |
| `quality_summary` | `results/screenshots/quality_summary_desktop_ck52vt6.png` | 将《三体》第一部中的黑暗森林法则压缩为文明竞争、隐藏与威慑等核心意思 | 摘要较短，能抓住主题，但“第一部”与完整黑暗森林概念存在一定概括化 |
| `quality_code` | `results/screenshots/quality_code_desktop_ck52vt6.png` | 正确解释递归 Fibonacci，并指出时间复杂度为指数级 `O(2^n)` | 代码解释准确，能指出重复子问题造成复杂度高 |
| `quality_reasoning` | `results/screenshots/quality_reasoning_desktop_ck52vt6.png` | 对宝石盒子逻辑题进行了分情况讨论 | 结论错误：在“仅一个陈述为真”的条件下，宝石应在盒子 2；模型最后判断“不在任何盒子”不符合题设 |
| `quality_osh` | `results/screenshots/quality_osh_desktop_ck52vt6.png` | 用三句话解释进程调度，说明其负责 CPU 资源分配、进程运行顺序和时间片安排 | 输出相关、简洁，满足课程相关问题要求 |

分析重点：

脚本质量评估结果来自 `results/raw/llama_cli_benchmark_20260605_230905.jsonl`：`quality_deterministic` 平均耗时 5.29s，`quality_more_diverse` 平均耗时 6.06s，二者最大 RSS 均约为 1.79GB。温度从 0.2 提高到 0.8 后，输出会更发散，端到端耗时略有上升，但主要性能差异仍来自生成 token 数量和 prompt 内容，而不是采样参数本身。

质量结论：

1. 中文问答、摘要、代码解释和课程相关问题均能得到可用回答。
2. 逻辑推理题暴露了小模型的可靠性问题，报告和应用中不能只看语言流畅度，还需要检查推理链和最终结论。
3. 线程、batch、ctx 等性能参数通常不改变模型权重，本身不应显著改变语义质量；采样温度、top-p 和重复惩罚更容易影响输出稳定性。

## 7. RPC 多机推理结果

### 7.1 拓扑与部署

本次 RPC 分布式推理的主机和从机不在同一台物理机器上：主机是连接手机热点的另一台电脑中的 WSL2，运行 `llama-cli`；从机是另一台电脑中的 VMware Ubuntu 虚拟机，运行 `rpc-server`。虚拟机最初拿到过 `192.168.247.x` 的 VMware NAT 地址，该地址只在从机 Windows 宿主机内部可达，不能被热点中的另一台电脑直接访问。因此最终将 VMware 网络改为桥接到热点 Wi-Fi 网卡，使虚拟机获得热点局域网地址 `10.210.218.47`。

从机启动命令：

```bash
cd Lab4
source config/experiment.env
RPC_EXTRA_ARGS="-H 0.0.0.0 -t 4" RPC_PORT=50052 ./scripts/start_rpc_server.sh
```

主机连接命令：

```bash
cd Lab4
source config/experiment.env
RPC_SERVERS="10.210.218.47:50052" \
PROMPT="请解释 llama.cpp RPC 后端为什么可能受网络延迟影响。" \
./scripts/run_rpc_inference.sh
```

实测截图：

| 截图 | 内容 |
| --- | --- |
| `results/screenshots/rpc_worker_server_vm_c6h14.png` | 从机 `rpc-server` 绑定 `0.0.0.0:50052`，使用 CPU 后端，并接收到主机连接 |
| `results/screenshots/rpc_host_inference_desktop_ck52vt6.png` | 主机 `llama-cli` 通过 `--rpc 10.210.218.47:50052` 完成推理 |

拓扑表：

| 节点 | IP | 角色 | 后端 | 命令 |
| --- | --- | --- | --- | --- |
| host / `DESKTOP-CK52VT6` | 热点局域网地址未单独截图记录 | 主机 `llama-cli`，加载本地 GGUF 模型并连接 RPC 后端 | WSL2 CPU | `docs/commands.md` 第 8 节 |
| worker-a / `c6h14-VMware-Virtual-Platform` | `10.210.218.47` | 从机 `rpc-server`，提供远程计算后端 | VMware Ubuntu CPU，无 CUDA | `docs/commands.md` 第 7 节 |

### 7.2 RPC 实测结果

| 模式 | Prompt/任务 | 总延迟 s | Prompt eval t/s | Decode / Generation t/s | 网络 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| 单机 `llama-bench` | `pp512` / `tg128` | 不适用 | `199.57 ± 6.20` | `38.42 ± 3.65` | 无 RPC | CPU 单机基线，见 `results/raw/llama_bench_20260605_231015_fixed.txt` |
| RPC 1 从机 | “请解释 llama.cpp RPC 后端为什么可能受网络延迟影响。” | 截图未单独记录 wall time | `27.4` | `12.3` | 手机热点，worker `10.210.218.47:50052` | 成功完成分布式推理，见 `rpc_host_inference_desktop_ck52vt6.png` |

说明：单机 `llama-bench` 与 RPC 截图使用的 prompt 和统计方式不同，数值不能作为严格同配置对比；但 RPC 生成吞吐低于单机 CPU 基线，足以说明当前手机热点与 CPU 从机组合没有带来加速收益。

RPC 分析要点：

1. `rpc-server` 必须绑定 `0.0.0.0`，否则默认只监听本机回环地址，主机无法从热点局域网连入。
2. VMware NAT 地址 `192.168.247.x` 不适用于本实验拓扑，因为主机位于另一台电脑的 WSL2 中，不能直接访问从机 Windows 内部的 NAT 网段；桥接后获得的 `10.210.218.47` 才是可达地址。
3. 从机没有 CUDA，实际使用 CPU 后端。截图中 `ggml_cuda_init: failed to initialize CUDA: no CUDA-capable device is detected` 是预期现象，不影响 CPU RPC 后端启动。
4. 手机热点的链路延迟、带宽和抖动弱于有线局域网；同时 Qwen2.5-1.5B Q4_K_M 模型较小，单机 CPU 已能较快推理，RPC 的通信和同步开销容易超过远端计算收益。
5. 当前结果说明 RPC 已经完成主从协同推理，但不是性能最优部署。更可能受益的场景是模型更大、单机内存不足、从机有更强 GPU，或网络换成稳定有线局域网。

## 8. Ray 批量推理结果

### 8.1 部署方式

Ray 选择性必做任务采用“Head A 本地 Ray 集群 + 两个 llama-server endpoint”的部署方式。Ray head 运行在主机 `DESKTOP-CK52VT6` 的 WSL2 内；两个 HTTP 推理服务分别是主机 WSL 本地 `llama-server` 和从机 VMware Ubuntu 的 `llama-server`。Ray Task 负责把 30 个 prompt 分发到两个 HTTP endpoint。

配置文件记录在 `config/ray_servers.final.json`：

| 节点 | URL | 角色 | 说明 |
| --- | --- | --- | --- |
| `head-a-wsl` | `http://127.0.0.1:8080` | Ray head 所在主机的本地 `llama-server` | 串行基线只使用该节点 |
| `worker-vm-c6h14` | `http://10.210.218.47:8080` | 从机 VMware Ubuntu 的 `llama-server` | 通过手机热点访问 |

实测截图：

| 截图 | 内容 |
| --- | --- |
| `results/screenshots/ray_status.png` | `ray status` 显示 1 个 active Ray 节点，资源为 20 CPU、9.33 GiB memory |
| `results/screenshots/ray_host.png` | 主机 `llama-server` 在 Ray 批量任务中处理请求，生成吞吐约 19-23 t/s |
| `results/screenshots/ray_workers.png` | 从机 `llama-server` 在 Ray 批量任务中处理请求，生成吞吐约 5.8 t/s |

### 8.2 批量推理结果

运行命令见 `docs/ray_task.md`，原始结果见 `results/raw/ray_serial.jsonl`、`results/raw/ray_round_robin.jsonl` 和 `results/raw/ray_summary.md`。

| 模式 | Prompt 数 | 总耗时 s | 平均延迟 s | P95 延迟 s | 吞吐 req/s | 失败数 | 说明 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 串行 | 30 | 105.471 | 3.516 | 4.600 | 0.284 | 0 | 单进程逐个请求，只访问 `head-a-wsl` |
| Ray 轮询 | 30 | 89.563 | 27.955 | 74.978 | 0.335 | 0 | Ray Task 并发提交，`head-a-wsl` 和 `worker-vm-c6h14` 各 15 个请求 |

节点级延迟：

| 模式 | 节点 | 请求数 | 平均延迟 s | 最短/最长 s |
| --- | --- | --- | --- | --- |
| 串行 | `head-a-wsl` | 30 | 3.516 | 2.515/4.664 |
| Ray 轮询 | `head-a-wsl` | 15 | 10.305 | 2.822/17.269 |
| Ray 轮询 | `worker-vm-c6h14` | 15 | 45.604 | 16.659/76.113 |

Ray 分析要点：

1. Ray 轮询将 30 个请求并发分发到两个 server，总耗时从 105.471s 降到 89.563s，批处理耗时降低约 15.1%，吞吐从 0.284 req/s 提高到 0.335 req/s，提升约 17.8%。
2. Ray 轮询的平均单请求延迟和 P95 延迟明显高于串行，并不矛盾：串行模式每次只跑一个请求，而 Ray 模式把请求并发提交到两个 server，队列等待时间被计入单请求延迟。
3. 从机 `worker-vm-c6h14` 的平均延迟 45.604s，明显高于主机的 10.305s；截图中从机生成吞吐约 5.8 t/s，主机约 19-23 t/s，异构性能差异导致轮询调度出现慢节点拖尾。
4. 手机热点带来的网络抖动和从机 VMware 虚拟化开销会进一步放大慢请求尾延迟，尤其是输出 token 数较长的 prompt。
5. 更合理的改进方向是按节点性能设置权重，减少从机分配比例，或采用延迟感知调度，把新请求优先发给历史延迟更低且当前 in-flight 更少的 server。

## 9. 结论

单机阶段结论：

1. 在 `DESKTOP-CK52VT6` 上，Qwen2.5-1.5B-Instruct Q4_K_M 可以稳定完成 CPU 单机推理，模型常驻内存约 1.8GB。
2. 当前最合适的单机配置是 `--threads 8 --batch-size 256 --ctx-size 2048 --n-gpu-layers 0`，5 个 prompt 的平均端到端耗时为 5.31s。
3. `llama-bench` 显示该模型在 CPU 后端下 prompt eval 约 199.57 t/s，decode 约 38.42 t/s。
4. 12 线程配置在 WSL2 中明显变慢，说明线程数并非越高越好，需要结合物理核心、调度环境和内存带宽观察。
5. 质量评估中代码解释和课程问答表现较好，但逻辑推理题出现错误，后续分析需要同时关注性能和输出正确性。

RPC 阶段结论：

1. 主机 `DESKTOP-CK52VT6` 成功通过手机热点连接 VMware 从机 `10.210.218.47:50052`，完成 llama.cpp RPC 分布式推理。
2. 从机 `rpc-server` 运行在 CPU 后端，无 CUDA；服务端截图显示多次 `Accepted client connection`，证明主机请求确实到达从机。
3. RPC 推理截图显示 Prompt eval 约 27.4 t/s，Generation 约 12.3 t/s，低于单机 CPU `llama-bench` 的 decode 约 38.42 t/s。主要原因是手机热点网络开销、从机 CPU 后端性能有限，以及小模型本身不足以摊薄 RPC 通信成本。
4. 本次实验验证了 llama.cpp 的 RPC 部署链路、网络可达性、远程后端连接和推理正确性；性能上不追求加速，而是体现分布式推理系统中网络、异构硬件和任务划分的影响。

Ray 阶段结论：

1. Ray 必做任务已完成 30 个 prompt 的串行基线与 Ray 轮询并发调度对比，两个模式均 30/30 成功、0 失败。
2. Ray 轮询提高了批处理吞吐和总耗时表现，但因为从机明显慢于主机，单请求平均延迟和 P95 延迟变差。
3. 该结果符合异构多机调度的常见现象：并发可以改善整体吞吐，但固定轮询在慢节点存在时会产生尾延迟。报告中因此不把 Ray 轮询描述为单请求加速，而是描述为批量吞吐优化。
