# Lab4 实验四提交目录

本目录用于提交实验四：llama.cpp 主线任务与 Ray 选择性必做任务。当前目录提供了可复现实验的文档、脚本、配置模板、命令记录模板和结果归档结构。模型文件、llama.cpp 源码构建目录以及本地运行指南不纳入提交。

## 目录结构

| 路径 | 内容 |
| --- | --- |
| `docs/deployment.md` | llama.cpp、RPC 和 Ray 的部署说明 |
| `docs/performance_analysis.md` | 性能指标、测试设计、数据表格和系统分析 |
| `docs/ray_task.md` | Ray 选择性必做任务说明 |
| `docs/commands.md` | 实验命令记录 |
| `config/*.example.*` | 本机和多机实验配置模板 |
| `data/prompts_quality.jsonl` | 5 个质量评估 prompt |
| `data/prompts_batch.jsonl` | 30 个 Ray 批量推理 prompt |
| `scripts/` | 构建、部署、测试、汇总脚本 |
| `results/raw/` | 原始输出、JSONL/CSV 测试结果 |
| `results/screenshots/` | 截图归档目录 |

## 交付项对应关系

| 实验要求 | 对应文件 |
| --- | --- |
| 部署说明文档 | `docs/deployment.md` |
| 性能测试与系统分析文档 | `docs/performance_analysis.md` |
| 实验脚本 | `scripts/` |
| 命令记录 | `docs/commands.md` |
| 配置文件 | `config/` |
| 结果截图 | `results/screenshots/` |
| Ray 选择性必做任务说明 | `docs/ray_task.md` |

## 当前状态说明

当前已完成 llama.cpp 主线任务的单机推理、参数扫描、质量评估和 RPC 分布式推理，也完成了 Ray 选择性必做任务的串行基线与 Ray 轮询批量推理对比。单机原始数据位于 `results/raw/llama_cli_benchmark_20260605_230453.jsonl` 与 `results/raw/llama_bench_20260605_231015_fixed.txt`；RPC 记录位于 `results/raw/rpc_inference_20260606.md`；Ray 记录位于 `results/raw/ray_serial.jsonl`、`results/raw/ray_round_robin.jsonl` 和 `results/raw/ray_summary.md`。

本地运行指南位于 `Lab4/LOCAL_RUN_GUIDE.md`，已写入 `.gitignore`，不要提交。
