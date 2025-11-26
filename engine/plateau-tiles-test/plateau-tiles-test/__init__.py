import shutil
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