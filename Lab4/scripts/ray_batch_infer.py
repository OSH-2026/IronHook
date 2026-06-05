#!/usr/bin/env python3
import argparse
import json
import statistics
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def read_jsonl(path):
    rows = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def write_jsonl(path, rows):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")


def request_path(endpoint_type):
    if endpoint_type == "openai-chat":
        return "/v1/chat/completions"
    return "/completion"


def make_payload(endpoint_type, prompt, request_cfg):
    n_predict = int(prompt.get("n_predict", request_cfg.get("n_predict", 128)))
    temperature = float(request_cfg.get("temperature", 0.2))
    if endpoint_type == "openai-chat":
        return {
            "model": request_cfg.get("model", "local-gguf"),
            "messages": [{"role": "user", "content": prompt["prompt"]}],
            "max_tokens": n_predict,
            "temperature": temperature,
            "stream": False,
        }
    return {
        "prompt": prompt["prompt"],
        "n_predict": n_predict,
        "temperature": temperature,
        "stream": False,
    }


def extract_text_and_tokens(endpoint_type, data):
    text = ""
    output_tokens = None
    if endpoint_type == "openai-chat":
        choices = data.get("choices") or []
        if choices:
            message = choices[0].get("message") or {}
            text = message.get("content", "")
        usage = data.get("usage") or {}
        output_tokens = usage.get("completion_tokens")
    else:
        text = data.get("content", "")
        timings = data.get("timings") or {}
        output_tokens = timings.get("predicted_n") or data.get("tokens_predicted")
    return text, output_tokens


def call_server(server, prompt, request_cfg, mode):
    endpoint_type = request_cfg.get("endpoint_type", "completion")
    timeout = float(request_cfg.get("request_timeout_s", 180))
    path = server.get("path") or request_path(endpoint_type)
    url = server["url"].rstrip("/") + path
    payload = make_payload(endpoint_type, prompt, request_cfg)
    body = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    start_time = utc_now()
    start = time.perf_counter()
    status = "ok"
    error = ""
    response_data = {}
    response_text = ""
    output_tokens = None
    http_status = None

    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            http_status = resp.status
            raw = resp.read().decode("utf-8", errors="replace")
            response_data = json.loads(raw) if raw else {}
            response_text, output_tokens = extract_text_and_tokens(endpoint_type, response_data)
    except urllib.error.HTTPError as exc:
        status = "error"
        http_status = exc.code
        error = exc.read().decode("utf-8", errors="replace")
    except Exception as exc:
        status = "error"
        error = repr(exc)

    latency_s = time.perf_counter() - start
    end_time = utc_now()
    if output_tokens is None:
        output_tokens = len(response_text.split())

    return {
        "mode": mode,
        "prompt_id": prompt["id"],
        "category": prompt.get("category", ""),
        "server": server["name"],
        "server_url": server["url"],
        "start_time": start_time,
        "end_time": end_time,
        "latency_s": latency_s,
        "status": status,
        "http_status": http_status,
        "error": error,
        "output_chars": len(response_text),
        "output_tokens_est": output_tokens,
        "response_text": response_text,
    }


def weighted_servers(servers):
    expanded = []
    for server in servers:
        weight = max(1, int(server.get("weight", 1)))
        expanded.extend([server] * weight)
    return expanded


def percentile(values, pct):
    if not values:
        return 0.0
    ordered = sorted(values)
    index = int(round((pct / 100) * (len(ordered) - 1)))
    return ordered[index]


def summarize(records, batch_elapsed_s):
    ok = [r for r in records if r["status"] == "ok"]
    latencies = [r["latency_s"] for r in ok]
    return {
        "total_requests": len(records),
        "ok_requests": len(ok),
        "failed_requests": len(records) - len(ok),
        "batch_elapsed_s": batch_elapsed_s,
        "avg_latency_s": statistics.mean(latencies) if latencies else 0.0,
        "p95_latency_s": percentile(latencies, 95),
        "throughput_req_s": (len(ok) / batch_elapsed_s) if batch_elapsed_s > 0 else 0.0,
        "requests_by_server": {
            name: sum(1 for r in records if r["server"] == name)
            for name in sorted({r["server"] for r in records})
        },
    }


def run_serial(prompts, servers, request_cfg):
    records = []
    server = servers[0]
    start = time.perf_counter()
    for prompt in prompts:
        records.append(call_server(server, prompt, request_cfg, "serial"))
    return records, time.perf_counter() - start


def init_ray(address):
    import ray

    if address and address != "local":
        ray.init(address=address, ignore_reinit_error=True)
    else:
        ray.init(ignore_reinit_error=True)
    return ray


def run_ray_static(mode, prompts, servers, request_cfg, address):
    ray = init_ray(address)
    remote_call = ray.remote(call_server)
    choices = weighted_servers(servers) if mode == "ray-weighted" else servers
    start = time.perf_counter()
    futures = []
    for index, prompt in enumerate(prompts):
        server = choices[index % len(choices)]
        futures.append(remote_call.remote(server, prompt, request_cfg, mode))
    records = ray.get(futures)
    return records, time.perf_counter() - start


def run_ray_latency_aware(prompts, servers, request_cfg, address, concurrency):
    ray = init_ray(address)
    remote_call = ray.remote(call_server)
    mode = "ray-latency-aware"
    avg_latency = {s["name"]: float(s.get("initial_latency_s", 1.0)) for s in servers}
    in_flight_count = {s["name"]: 0 for s in servers}
    pending_prompts = list(prompts)
    futures = []
    records = []

    def choose_server():
        return min(
            servers,
            key=lambda s: avg_latency[s["name"]] * (1 + in_flight_count[s["name"]]),
        )

    start = time.perf_counter()
    while pending_prompts or futures:
        while pending_prompts and len(futures) < concurrency:
            prompt = pending_prompts.pop(0)
            server = choose_server()
            in_flight_count[server["name"]] += 1
            future = remote_call.remote(server, prompt, request_cfg, mode)
            futures.append((future, server["name"]))

        ready, _ = ray.wait([future for future, _ in futures], num_returns=1)
        ready_set = set(ready)
        remaining = []
        for future, server_name in futures:
            if future in ready_set:
                result = ray.get(future)
                records.append(result)
                in_flight_count[server_name] -= 1
                if result["status"] == "ok":
                    avg_latency[server_name] = 0.7 * avg_latency[server_name] + 0.3 * result["latency_s"]
            else:
                remaining.append((future, server_name))
        futures = remaining

    return records, time.perf_counter() - start


def main():
    parser = argparse.ArgumentParser(description="Dispatch prompt batches to llama-server endpoints.")
    parser.add_argument("--mode", required=True, choices=["serial", "ray-round-robin", "ray-weighted", "ray-latency-aware"])
    parser.add_argument("--config", required=True)
    parser.add_argument("--prompts", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--summary-out", default="")
    parser.add_argument("--ray-address", default="local")
    parser.add_argument("--concurrency", type=int, default=4)
    args = parser.parse_args()

    with open(args.config, "r", encoding="utf-8") as f:
        request_cfg = json.load(f)
    servers = request_cfg["servers"]
    prompts = read_jsonl(args.prompts)

    if args.mode == "serial":
        records, batch_elapsed_s = run_serial(prompts, servers, request_cfg)
    elif args.mode in ("ray-round-robin", "ray-weighted"):
        records, batch_elapsed_s = run_ray_static(args.mode, prompts, servers, request_cfg, args.ray_address)
    else:
        records, batch_elapsed_s = run_ray_latency_aware(
            prompts,
            servers,
            request_cfg,
            args.ray_address,
            max(1, args.concurrency),
        )

    summary = summarize(records, batch_elapsed_s)
    for record in records:
        record["batch_summary"] = summary

    write_jsonl(args.out, records)
    summary_out = args.summary_out or str(Path(args.out).with_suffix(".summary.json"))
    Path(summary_out).write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    print(f"Wrote {args.out}")
    print(f"Wrote {summary_out}")


if __name__ == "__main__":
    main()
