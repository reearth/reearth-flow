import sys

def stderrLog(prefix, *args, **kwargs):
    return lambda *args, **kwargs: print(prefix, *args, **kwargs, file=sys.stderr)

info = stderrLog("[INFO]")

def quiet():
    global info
    info = lambda *args, **kwargs: None