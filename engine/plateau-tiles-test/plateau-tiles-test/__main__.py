import sys, os, shutil
from .runner import run_testcase
from . import BASE_PATH, cleanup

# tests when no arguments are given (for CI)
default_tests = [
    "13115_suginami-ku_conv-tran_multipolygon",
    "14212_atsugi-shi_conv-tran_squr",
    "08220_tsukuba-shi_conv-tran_dm",
    "16202_takaoka-shi_conv-tran_rwy",
    "22203_numazu-shi_conv-tran_wwy",
    "08220_tsukuba-shi_conv-tran_3dtiles",
]
default_stages = "re"

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