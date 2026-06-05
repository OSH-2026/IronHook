#!/usr/bin/env python3
import argparse
import json
import statistics
from pathlib import Path


def read_jsonl(path):
    rows = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def percentile(values, pct):
    if not values:
        return 0.0
    ordered = sorted(values)
    index = int(round((pct / 100) * (len(ordered) - 1)))
    return ordered[index]


def summarize(path):
    rows = read_jsonl(path)
    ok = [r for r in rows if r.get("status") == "ok" or r.get("returncode") == 0]
    latencies = []
    for row in ok:
        if "latency_s" in row:
            latencies.append(float(row["latency_s"]))
        elif "elapsed_s" in row:
            latencies.append(float(row["elapsed_s"]))

    first_summary = rows[0].get("batch_summary") if rows else None
    batch_elapsed = 0.0
    throughput = 0.0
    if first_summary:
        batch_elapsed = float(first_summary.get("batch_elapsed_s", 0.0))
        throughput = float(first_summary.get("throughput_req_s", 0.0))
    elif latencies:
        batch_elapsed = sum(latencies)
        throughput = len(ok) / batch_elapsed if batch_elapsed else 0.0

    servers = {}
    for row in rows:
        server = row.get("server") or row.get("config") or "unknown"
        servers[server] = servers.get(server, 0) + 1

    return {
        "file": str(path),
        "mode": rows[0].get("mode", rows[0].get("config", "unknown")) if rows else "unknown",
        "total": len(rows),
        "ok": len(ok),
        "failed": len(rows) - len(ok),
        "batch_elapsed_s": batch_elapsed,
        "avg_latency_s": statistics.mean(latencies) if latencies else 0.0,
        "p95_latency_s": percentile(latencies, 95),
        "throughput_req_s": throughput,
        "servers": servers,
    }


def render_markdown(summaries):
    lines = [
        "# 实验结果汇总",
        "",
        "| 文件 | 模式 | 总请求 | 成功 | 失败 | 总耗时 s | 平均延迟 s | P95 延迟 s | 吞吐 req/s |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for item in summaries:
        lines.append(
            "| {file} | {mode} | {total} | {ok} | {failed} | {batch_elapsed_s:.3f} | "
            "{avg_latency_s:.3f} | {p95_latency_s:.3f} | {throughput_req_s:.3f} |".format(**item)
        )
    lines.append("")
    lines.append("## 节点请求数")
    lines.append("")
    for item in summaries:
        lines.append(f"### {item['file']}")
        lines.append("")
        lines.append("| 节点/配置 | 请求数 |")
        lines.append("| --- | --- |")
        for server, count in sorted(item["servers"].items()):
            lines.append(f"| {server} | {count} |")
        lines.append("")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Summarize JSONL benchmark results.")
    parser.add_argument("paths", nargs="+")
    parser.add_argument("--out", default="")
    args = parser.parse_args()

    summaries = [summarize(Path(path)) for path in args.paths]
    markdown = render_markdown(summaries)
    if args.out:
        Path(args.out).write_text(markdown, encoding="utf-8")
        print(f"Wrote {args.out}")
    else:
        print(markdown)


if __name__ == "__main__":
    main()
