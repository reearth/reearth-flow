import sys

def stderrLog(prefix):
    return lambda *args, **kwargs: print(prefix, *args, **kwargs, file=sys.stderr)

info = stderrLog("[INFO]")
debug = stderrLog("[DEBUG]")

def quiet():
    global debug
    debug = lambda *args, **kwargs: None