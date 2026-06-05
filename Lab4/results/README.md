# 结果目录说明

`raw/` 保存脚本生成的原始结果，例如：

| 文件 | 来源 |
| --- | --- |
| `env_*.md` | `scripts/collect_env.sh` |
| `llama_cli_benchmark_*.jsonl` | `scripts/llama_cli_benchmark.py` |
| `ray_*.jsonl` | `scripts/ray_batch_infer.py` |
| `ray_summary.md` | `scripts/summarize_results.py` |

`screenshots/` 保存终端、Ray Dashboard、RPC 推理成功等截图。截图不需要由脚本生成，但需要能证明命令和结果确实运行。
