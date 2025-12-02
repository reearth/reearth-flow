import os, shutil, subprocess, time
import tomllib, zipfile
from pathlib import Path
from . import BASE_PATH, ENGINE_PATH, reset_dir, cleanup, log
from .filter import filter_zip, extract_zip_to_structure
from .align_mvt import test_mvt_attributes, test_mvt_lines, test_mvt_polygons
from .align_3dtiles import test_3dtiles_attributes
from .compare_attributes import test_json_attributes

def pack_citymodel_zip(zip_stem, testcase_dir, artifacts_base, output_path):
    """Pack zip from artifacts (codelists/schemas) + testcase citymodel/ overrides."""
    artifact_dir = artifacts_base / zip_stem
    testcase_citymodel = testcase_dir / "citymodel"

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output_path, 'w', zipfile.ZIP_DEFLATED) as zf:
        # Add codelists/ and schemas/ from artifacts
        for dirname in ["codelists", "schemas"]:
            src = artifact_dir / dirname
            if src.exists():
                for f in src.rglob("*"):
                    if f.is_file():
                        zf.write(f, f"{dirname}/{f.relative_to(src)}")

        # Add testcase citymodel/ files (overrides)
        if testcase_citymodel.exists():
            for f in testcase_citymodel.rglob("*"):
                if f.is_file():
                    zf.write(f, str(f.relative_to(testcase_citymodel)))

def extract_fme_output(fme_zip_path, fme_dir):
    fme_dir.parent.mkdir(parents=True, exist_ok=True)

    needs_extract = True
    if fme_dir.exists():
        fme_zip_mtime = fme_zip_path.stat().st_mtime
        fme_dir_files = [p for p in fme_dir.rglob("*") if p.is_file()]
        if fme_dir_files:
            fme_dir_mtime = max(p.stat().st_mtime for p in fme_dir_files)
            if fme_dir_mtime >= fme_zip_mtime:
                needs_extract = False

    if needs_extract:
        reset_dir(fme_dir)
        log.debug("Extracting FME output:", fme_zip_path, "->", fme_dir)
        with zipfile.ZipFile(fme_zip_path, 'r') as zip_ref:
            zip_ref.extractall(fme_dir)
        for mvt_file in fme_dir.rglob("*.mvt"):
            mvt_file.rename(mvt_file.with_suffix(".pbf"))

def run_workflow(workflow_path, output_dir, citygml_path):
    env = os.environ.copy()
    env['FLOW_VAR_cityGmlPath'] = str(citygml_path)
    if citygml_path.suffix == ".gml":
        # single file mode, search for codelists and schemas
        d = citygml_path.parent
        while d != d.parent:
            if (d / "codelists").is_dir() and (d / "schemas").is_dir():
                env["FLOW_VAR_codelists"] = d / "codelists"
                env["FLOW_VAR_schemas"] = d / "schemas"
                break
            d = d.parent
        else:
            raise FileNotFoundError("codelists and schemas directories not found")
    workflow_path = ENGINE_PATH / workflow_path
    if not cleanup:
        # save current workflow as JSON for reference
        import yaml, json
        result = subprocess.run(["yaml-include", str(workflow_path)], capture_output=True, text=True, check=True)
        workflow_json = json.dumps(yaml.safe_load(result.stdout), indent=2)
        with open(output_dir / "workflow.json", "w") as wf:
            wf.write(workflow_json)
    env['FLOW_EXAMPLE_TARGET_WORKFLOW'] = str(workflow_path)
    flow_dir = reset_dir(output_dir / "flow")
    env['FLOW_VAR_workerArtifactPath'] = str(flow_dir)
    runtime_dir = reset_dir(output_dir / "runtime")
    env['FLOW_RUNTIME_WORKING_DIRECTORY'] = str(runtime_dir)
    process = subprocess.Popen(
        ["cargo", "run", "--example", "example_main"],
        cwd=ENGINE_PATH,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    for line in process.stdout:
        log.debug(line.rstrip())
    process.wait()
    if process.returncode != 0:
        raise subprocess.CalledProcessError(process.returncode, process.args)

def run_testcase(path, stages):
    # Preserve structure: testcases/{workflow-path}/{desc} -> results/{workflow-path}/{desc}
    relative_path = path.relative_to(BASE_PATH / "testcases")
    output_dir = BASE_PATH / "results" / relative_path
    with open(path / "profile.toml", "rb") as f:
        profile = tomllib.load(f)

    # Derive workflow_path from testcase directory structure
    # testcases/{category}/{version}/{workflow-id}/{desc} -> workflow/{category}/{version}/{workflow-id}/workflow.yml
    if "workflow_path" in profile:
        # Use explicit workflow_path from profile if provided (override)
        workflow_path = ENGINE_PATH / profile["workflow_path"]
    else:
        # Derive from path structure: remove last part (desc) to get workflow path
        workflow_parts = relative_path.parts[:-1]
        workflow_path = ENGINE_PATH / "runtime/examples/fixture/workflow" / Path(*workflow_parts) / "workflow.yml"

    citygml_zip_name = profile["citygml_zip_name"]
    zip_stem = citygml_zip_name.removesuffix(".zip")
    artifacts_base = BASE_PATH / "artifacts" / "citymodel"
    citygml_srcdir = Path(os.environ.get("CITYGML_SRCDIR", ".")).resolve()
    citygml_path = output_dir / citygml_zip_name

    if "g" in stages:
        extract_zip_to_structure(
            citygml_srcdir / citygml_zip_name,
            artifacts_base,
            path,
            profile.get("filter", {}).get("tree", {}),
            zip_stem
        )
        pack_citymodel_zip(zip_stem, path, artifacts_base, citygml_path)

    fme_output_path = path / "fme.zip"
    if "r" in stages:
        if not citygml_path.exists():
            pack_citymodel_zip(zip_stem, path, artifacts_base, citygml_path)
        log.info(f"Starting run: {relative_path}")
        reset_dir(output_dir / "flow")
        reset_dir(output_dir / "runtime")
        start_time = time.time()
        run_workflow(workflow_path, output_dir, citygml_path)
        elapsed = time.time() - start_time
        log.info(f"Completed run: {relative_path} ({elapsed:.2f}s)")
    if "e" in stages:
        assert fme_output_path.exists(), "FME output file not found in testcase"
        fme_dir = output_dir / "fme"
        extract_fme_output(fme_output_path, fme_dir)
        for test_name, cfg in profile.get("tests", {}).items():
            log.info(f"Starting test: {relative_path}/{test_name}")
            start_time = time.time()
            if test_name == "mvt_attributes":
                test_mvt_attributes(fme_dir, output_dir / "flow", cfg)
            elif test_name == "mvt_lines":
                test_mvt_lines(fme_dir, output_dir / "flow", cfg)
            elif test_name == "mvt_polygons":
                test_mvt_polygons(fme_dir, output_dir / "flow", cfg)
            elif test_name == "3dtiles_attributes":
                test_3dtiles_attributes(fme_dir, output_dir / "flow", cfg)
            elif test_name == "json_attributes":
                test_json_attributes(fme_dir, output_dir / "flow", cfg)
            else:
                raise ValueError(f"Unknown test type: {test_name}")
            elapsed = time.time() - start_time
            log.info(f"Completed test: {relative_path}/{test_name} ({elapsed:.2f}s)")
    return output_dir