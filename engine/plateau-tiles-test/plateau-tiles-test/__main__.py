import sys, os, shutil
from .runner import run_testcase
from . import BASE_PATH
from .log import quiet

# tests when no arguments are given (for CI)
default_tests = [
    "14212-squr",
    # "08220-dm",
    "16202-rwy",
    "22203-wwy",
    "08220-3dtiles",
]
default_stages = "re"

if len(sys.argv) > 1:
    test = sys.argv[1]
    stages = sys.argv[2] if len(sys.argv) > 2 else default_stages
    path = BASE_PATH / "testcases" / test
    run_testcase(path, stages)
else:
    quiet()
    for test in default_tests:
        path = BASE_PATH / "testcases" / test
        output_dir = run_testcase(path, default_stages)
        if os.environ.get("CLEANUP_AFTER_TESTS", "0") == "1":
            # output_dir must exist or we should fail the test
            shutil.rmtree(output_dir)