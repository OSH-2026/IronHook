# 实验结果汇总

| 文件 | 模式 | 总请求 | 成功 | 失败 | 总耗时 s | 平均延迟 s | P95 延迟 s | 吞吐 req/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| results/raw/ray_round_robin.jsonl | ray-round-robin | 30 | 30 | 0 | 89.563 | 27.955 | 74.978 | 0.335 |
| results/raw/ray_serial.jsonl | serial | 30 | 30 | 0 | 105.471 | 3.516 | 4.600 | 0.284 |

## 节点请求数

### results/raw/ray_round_robin.jsonl

| 节点/配置 | 请求数 |
| --- | --- |
| head-a-wsl | 15 |
| worker-vm-c6h14 | 15 |

### results/raw/ray_serial.jsonl

| 节点/配置 | 请求数 |
| --- | --- |
| head-a-wsl | 30 |
