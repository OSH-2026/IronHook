#!/usr/bin/env python3
import argparse
import json
import os
import re
import shutil
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path


LOAD_RE = re.compile(r"load time\s*=\s*([0-9.]+)\s*ms")
PROMPT_RE = re.compile(
    r"(?:^|\n)[^\n]*prompt eval time\s*=\s*([0-9.]+)\s*ms\s*/\s*([0-9]+)\s*tokens?.*?([0-9.]+)\s*tokens per second",
    re.DOTALL,
)
EVAL_RE = re.compile(
    r"(?:^|\n)(?![^\n]*prompt eval)[^\n]*eval time\s*=\s*([0-9.]+)\s*ms\s*/\s*([0-9]+)\s*(?:runs|tokens?).*?([0-9.]+)\s*tokens per second",
    re.DOTALL,
)
RSS_RE = re.compile(r"Maximum resident set size \(kbytes\):\s*([0-9]+)")


def read_jsonl(path):
    rows = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def parse_metrics(stderr):
    metrics = {}
    load = LOAD_RE.search(stderr)
    prompt = PROMPT_RE.search(stderr)
    eval_match = EVAL_RE.search(stderr)
    rss = RSS_RE.search(stderr)
    if load:
        metrics["load_ms"] = float(load.group(1))
    if prompt:
        metrics["prompt_eval_ms"] = float(prompt.group(1))
        metrics["prompt_tokens"] = int(prompt.group(2))
        metrics["prompt_tokens_per_s"] = float(prompt.group(3))
    if eval_match:
        metrics["decode_ms"] = float(eval_match.group(1))
        metrics["decode_tokens"] = int(eval_match.group(2))
        metrics["decode_tokens_per_s"] = float(eval_match.group(3))
    if rss:
        metrics["max_rss_kb"] = int(rss.group(1))
    return metrics


def safe_name(value):
    return re.sub(r"[^A-Za-z0-9_.-]+", "_", value)


def run_one(args, config, prompt, out_dir, run_id, use_time):
    n_predict = int(prompt.get("n_predict", args.default_n_predict))
    command = [
        args.llama_bin,
        "-m",
        args.model,
        "-p",
        prompt["prompt"],
        "-n",
        str(n_predict),
    ] + list(config.get("args", []))

    wrapped = command
    if use_time:
        wrapped = ["/usr/bin/time", "-v"] + command

    started_at = datetime.now(timezone.utc).isoformat()
    start = time.perf_counter()
    try:
        proc = subprocess.run(
            wrapped,
            capture_output=True,
            text=True,
            timeout=args.timeout,
            check=False,
        )
        elapsed_s = time.perf_counter() - start
        returncode = proc.returncode
        stdout = proc.stdout
        stderr = proc.stderr
        error = ""
    except subprocess.TimeoutExpired as exc:
        elapsed_s = time.perf_counter() - start
        returncode = 124
        stdout = exc.stdout or ""
        stderr = exc.stderr or ""
        error = f"timeout after {args.timeout}s"

    ended_at = datetime.now(timezone.utc).isoformat()
    base = f"{run_id}_{safe_name(config['name'])}_{safe_name(prompt['id'])}"
    stdout_path = out_dir / f"{base}.stdout.txt"
    stderr_path = out_dir / f"{base}.stderr.txt"
    stdout_path.write_text(stdout, encoding="utf-8", errors="replace")
    stderr_path.write_text(stderr, encoding="utf-8", errors="replace")

    metrics = parse_metrics(stderr)
    return {
        "timestamp": started_at,
        "ended_at": ended_at,
        "config": config["name"],
        "prompt_id": prompt["id"],
        "category": prompt.get("category", ""),
        "elapsed_s": elapsed_s,
        "returncode": returncode,
        "error": error,
        "stdout_chars": len(stdout),
        "stderr_chars": len(stderr),
        "stdout_path": str(stdout_path),
        "stderr_path": str(stderr_path),
        "command": command,
        "metrics": metrics,
    }


def main():
    parser = argparse.ArgumentParser(description="Run llama-cli benchmark prompts.")
    parser.add_argument("--llama-bin", required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--prompts", required=True)
    parser.add_argument("--configs", required=True)
    parser.add_argument("--out-dir", default="results/raw")
    parser.add_argument("--timeout", type=int, default=600)
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--default-n-predict", type=int, default=128)
    parser.add_argument("--no-time", action="store_true", help="Do not wrap commands with /usr/bin/time -v.")
    args = parser.parse_args()

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    prompts = read_jsonl(args.prompts)
    if args.limit:
        prompts = prompts[: args.limit]

    with open(args.configs, "r", encoding="utf-8") as f:
        config_data = json.load(f)
    args.default_n_predict = int(config_data.get("default_n_predict", args.default_n_predict))
    configs = config_data["configs"]

    use_time = (not args.no_time) and os.path.exists("/usr/bin/time")
    run_id = datetime.now().strftime("llama_cli_benchmark_%Y%m%d_%H%M%S")
    jsonl_path = out_dir / f"{run_id}.jsonl"

    with jsonl_path.open("w", encoding="utf-8") as out:
        for config in configs:
            for prompt in prompts:
                record = run_one(args, config, prompt, out_dir, run_id, use_time)
                out.write(json.dumps(record, ensure_ascii=False) + "\n")
                out.flush()
                print(
                    f"{record['config']} {record['prompt_id']} "
                    f"returncode={record['returncode']} elapsed={record['elapsed_s']:.3f}s"
                )

    print(f"Wrote {jsonl_path}")


if __name__ == "__main__":
    main()
