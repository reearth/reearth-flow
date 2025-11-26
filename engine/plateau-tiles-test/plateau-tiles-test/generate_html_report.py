#!/usr/bin/env python3
import json, tomllib
import os, sys
import subprocess
from pathlib import Path
import yaml
from . import BASE_PATH

def resolve_workflow_to_json(workflow_path):
	result = subprocess.run(["yaml-include", str(workflow_path)], capture_output=True, text=True, check=True)
	return json.dumps(yaml.safe_load(result.stdout), indent=2)

def collect_edge_data(jobs_dir):
	if not jobs_dir.exists():
		return {}
	edge_data = {}
	for job_dir in jobs_dir.iterdir():
		if not job_dir.is_dir():
			continue
		feature_store = job_dir / "feature-store"
		if not feature_store.exists():
			continue
		for jsonl_file in feature_store.glob("*.jsonl"):
			parts = jsonl_file.stem.split(".")
			edge_id = parts[-1] if len(parts) > 1 else parts[0]
			features = []
			with open(jsonl_file, "r") as f:
				for line in f:
					line = line.strip()
					if line:
						feature = json.loads(line)
						# to save size
						feature.pop("geometry", None)
						features.append(feature)
			if features:
				edge_data[edge_id] = {
					"count": len(features),
					"features": features
				}
	return edge_data

def generate_html_report(output_dir, workflow_path):
	template_path = Path(__file__).parent / "workflow_template.html"
	html = open(template_path).read()

	workflow_json = resolve_workflow_to_json(workflow_path)
	open(output_dir / "workflow.json", "w").write(workflow_json)

	html = html.replace("{{WORKFLOW_JSON}}", workflow_json.replace("\\", "\\\\").replace("`", "\\`"))
	jobs_dir = output_dir / "runtime/projects/engine/jobs"
	edge_data = collect_edge_data(jobs_dir)
	html = html.replace("{{EDGE_DATA}}", json.dumps(edge_data).replace("\\", "\\\\").replace("`", "\\`"))

	open(output_dir / "workflow.html", "w").write(html)
	print(f"HTML report:", output_dir / "workflow.html")