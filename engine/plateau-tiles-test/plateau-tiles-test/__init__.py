import shutil, os, sys
from pathlib import Path

BASE_PATH = Path(__file__).parent.parent
ENGINE_PATH = BASE_PATH.parent

def reset_dir(path):
    try:
        shutil.rmtree(path)
    except FileNotFoundError:
        # ignore if the directory does not exist
        pass
    path.mkdir(parents=True, exist_ok=True)
    return path

class Logger:
    def __init__(self, verbosity=1):
        if isinstance(verbosity, str):
            self.verbosity = {"debug": 3, "info": 2, "warn": 1, "error": 0}.get(verbosity.lower(), 1)
        else:
            self.verbosity = int(verbosity)
    def debug(self, *args, **kwargs):
        if self.verbosity >= 3:
            print("[DEBUG]", *args, **kwargs, file=sys.stderr)
    def info(self, *args, **kwargs):
        if self.verbosity >= 2:
            print("[INFO]", *args, **kwargs, file=sys.stderr)
    def warn(self, *args, **kwargs):
        if self.verbosity >= 1:
            print("[WARN]", *args, **kwargs, file=sys.stderr)
    def error(self, *args, **kwargs):
        raise Exception("explicit panic", *args, **kwargs)

verbosity = os.environ.get("PLATEAU_TILES_TEST_VERBOSITY", "debug")
cleanup = os.environ.get("CLEANUP_AFTER_TEST", "0") == "1"

log = Logger(verbosity)