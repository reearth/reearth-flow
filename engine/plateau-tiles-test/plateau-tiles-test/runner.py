import os, shutil
import tomllib
import subprocess
import zipfile
from pathlib import Path
from . import BASE_PATH, ENGINE_PATH, reset_dir
from .filter import filter_zip
from .align_mvt import test_mvt_attributes
from .align_3dtiles import test_3dtiles_attributes
from . import log

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

def run_workflow(profile, output_dir, citygml_path):
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
    env['FLOW_EXAMPLE_TARGET_WORKFLOW'] = str(ENGINE_PATH / profile["workflow_path"])
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
    name = path.name
    output_dir = BASE_PATH / "results" / name
    with open(path / "profile.toml", "rb") as f:
        profile = tomllib.load(f)

    citygml_zip_name = profile["citygml_zip_name"]
    citygml_path = path / citygml_zip_name
    citygml_srcdir = Path(os.environ.get("CITYGML_SRCDIR", ".")).resolve()
    if "g" in stages:
        filter_zip(citygml_srcdir / citygml_zip_name, citygml_path, profile.get("filter", {}).get("tree", {}))
    fme_output_path = path / "fme.zip"
    if "r" in stages:
        log.info("running:", path.name)
        reset_dir(output_dir / "flow")
        reset_dir(output_dir / "runtime")
        run_workflow(profile, output_dir, citygml_path)
    if "e" in stages:
        log.info("evaluating:", name, "tests:", [key for key in profile.get("tests", {})])
        assert fme_output_path.exists(), "FME output file not found in testcase"
        fme_dir = output_dir / "fme"
        extract_fme_output(fme_output_path, fme_dir)
        for test, cfg in profile.get("tests", {}).items():
            if test == "mvt_attributes":
                test_mvt_attributes(fme_dir, output_dir / "flow", cfg)
            elif test == "3dtiles_attributes":
                test_3dtiles_attributes(fme_dir, output_dir / "flow", cfg)
            else:
                raise ValueError(f"Unknown test type: {test}")
    return output_dir