import sys

def stderr_log(prefix):
    return lambda *args, **kwargs: print(prefix, *args, **kwargs, file=sys.stderr)

info = stderr_log("[INFO]")
debug = stderr_log("[DEBUG]")

def quiet():
    global debug
    debug = lambda *args, **kwargs: None