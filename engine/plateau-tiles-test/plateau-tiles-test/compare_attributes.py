import json
from . import log

def dict_zip(dict1, dict2):
    keys = set(dict1.keys()).union(set(dict2.keys()))
    for k in keys:
        yield k, dict1.get(k, None), dict2.get(k, None)

def cast_attr(key, value, casts):
    if key not in casts:
        return value
    cast_type = casts[key]
    if cast_type == "string":
        return str(value)
    elif cast_type == "json":
        if isinstance(value, str):
            return json.loads(value)
        return value
    elif cast_type is None:
        return None
    else:
        raise ValueError(f"Unknown cast type: {cast_type}")

def compare_recurse(key, v1, v2, gid, bads, casts):
    v1 = cast_attr(key, v1, casts)
    v2 = cast_attr(key, v2, casts)

    if type(v1) != type(v2):
        if isinstance(v2, bool) and bool(v1) == v2:
            return
        if isinstance(v2, str) and str(v1) == v2:
            return
        bads.append((gid, key, v1, v2))
        return

    if isinstance(v1, dict):
        for k in set(v1.keys()).union(set(v2.keys())):
            compare_recurse(f"{key}.{k}", v1.get(k), v2.get(k), gid, bads, casts)
    elif isinstance(v1, list):
        if len(v1) != len(v2):
            bads.append((gid, key, v1, v2))
            return
        for idx in range(len(v1)):
            compare_recurse(f"{key}[{idx}]", v1[idx], v2[idx], gid, bads, casts)
    else:
        if v1 != v2:
            bads.append((gid, key, v1, v2))

def analyze_attributes(gml_id, attr1, attr2, casts):
    if attr1 is None or attr2 is None:
        raise ValueError(f"Missing attributes for gml_id: {gml_id}")

    bads = []
    compare_recurse("", attr1, attr2, gml_id, bads, casts)

    if bads:
        for gid, k, v1, v2 in bads:
            print(f"MISMATCH gml_id={gid} key={k} v1={repr(v1)} v2={repr(v2)}")
        raise ValueError(f"Attribute mismatches found for gml_id: {gml_id}")
    else:
        log.debug(f"Attributes match for gml_id: {gml_id}")
