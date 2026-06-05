# llama.cpp RPC 推理记录

日期：2026-06-06

## 拓扑

| 节点 | 记录 |
| --- | --- |
| 主机 | `DESKTOP-CK52VT6`，WSL2，运行 `llama-cli` |
| 从机 | `c6h14-VMware-Virtual-Platform`，VMware Ubuntu 虚拟机，运行 `rpc-server` |
| 网络 | 手机热点局域网 |
| 从机地址 | `10.210.218.47:50052` |
| 模型 | `qwen2.5-1.5b-instruct-q4_k_m.gguf` |

## 从机命令

```bash
cd Lab4
source config/experiment.env
RPC_EXTRA_ARGS="-H 0.0.0.0 -t 4" RPC_PORT=50052 ./scripts/start_rpc_server.sh
```

从机截图显示 `rpc-server` 监听 `0.0.0.0:50052`，使用 CPU 后端，并多次接收主机连接。启动时出现 `ggml_cuda_init: failed to initialize CUDA: no CUDA-capable device is detected`，原因是虚拟机没有 CUDA 设备，本次实验按 CPU 后端运行。

## 主机命令

```bash
cd Lab4
source config/experiment.env
RPC_SERVERS="10.210.218.47:50052" \
PROMPT="请解释 llama.cpp RPC 后端为什么可能受网络延迟影响。" \
./scripts/run_rpc_inference.sh
```

## 结果

| 指标 | 数值 |
| --- | --- |
| Prompt eval | `27.4 t/s` |
| Generation | `12.3 t/s` |
| 结果截图 | `results/screenshots/rpc_host_inference_desktop_ck52vt6.png` |
| 从机截图 | `results/screenshots/rpc_worker_server_vm_c6h14.png` |

## 说明

虚拟机最初出现过 `192.168.247.x` 地址，这是 VMware NAT 网段，热点中的另一台电脑无法直接访问。最终将虚拟机网络改为桥接到手机热点对应的 Wi-Fi 网卡，使虚拟机获得 `10.210.218.47`，主机再通过该地址连接 RPC 后端。
