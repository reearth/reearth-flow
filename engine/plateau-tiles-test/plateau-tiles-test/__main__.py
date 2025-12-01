import sys, os, shutil
from .runner import run_testcase
from . import BASE_PATH, cleanup

# tests when no arguments are given (for CI)
default_tests = [
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/multipolygon",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/squr",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/dm",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/rwy",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/wwy",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/3dtiles",
    "data-convert/plateau4/06-area-urf/urf",
]
default_stages = os.environ.get("PLATEAU_TILES_TEST_STAGES", "re")

if len(sys.argv) > 1:
    name = sys.argv[1]
    stages = sys.argv[2] if len(sys.argv) > 2 else default_stages
    path = BASE_PATH / "testcases" / name
    run_testcase(path, stages)
else:
    for name in default_tests:
        path = BASE_PATH / "testcases" / name
        output_dir = run_testcase(path, default_stages)
        if cleanup:
            # output_dir must exist or we should fail the test
            shutil.rmtree(output_dir)