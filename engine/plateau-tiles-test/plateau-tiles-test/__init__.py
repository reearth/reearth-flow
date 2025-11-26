import shutil
from pathlib import Path

BASE_PATH = Path(__file__).parent.parent
ENGINE_PATH = BASE_PATH.parent

def reset_dir(path, create = True):
    try:
        shutil.rmtree(path)
    except FileNotFoundError:
        pass
    path.mkdir(parents=True, exist_ok=True)
    return path