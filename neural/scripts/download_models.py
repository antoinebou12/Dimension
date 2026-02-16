#!/usr/bin/env python3
"""
Download ONNX models into neural/models/ for use by the neural crate.
Requires: pip install huggingface_hub

Usage:
  python download_models.py [--graph] [--all]
  From repo root: python neural/scripts/download_models.py --all
"""
from __future__ import annotations

import argparse
import os
import sys


def main() -> int:
    parser = argparse.ArgumentParser(description="Download ONNX models into neural/models/")
    parser.add_argument("--graph", action="store_true", help="Download graph embedding ONNX (Hugging Face)")
    parser.add_argument("--all", action="store_true", help="Download all supported models")
    args = parser.parse_args()

    if not args.graph and not args.all:
        parser.print_help()
        return 0

    try:
        from huggingface_hub import hf_hub_download
    except ImportError:
        print("huggingface_hub required: pip install huggingface_hub", file=sys.stderr)
        return 1

    # neural/models/ relative to repo root; script may be run from repo root or neural/scripts/
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.dirname(os.path.dirname(script_dir))
    models_dir = os.path.join(repo_root, "neural", "models")
    os.makedirs(models_dir, exist_ok=True)

    if args.graph or args.all:
        # Knowledge-graph / graph NLP ONNX from Hugging Face
        repo_id = "vishnun/quantized_knowledge_graph_nlp_onnx"
        filename = "model_quantized.onnx"
        local_file = hf_hub_download(
            repo_id=repo_id,
            filename=filename,
            local_dir=models_dir,
            local_dir_use_symlinks=False,
        )
        print(f"Downloaded graph ONNX -> {local_file}")

    print(f"Models dir: {models_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
