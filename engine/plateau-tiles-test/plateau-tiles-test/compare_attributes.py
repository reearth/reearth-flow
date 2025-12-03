import json
from . import log

def dict_zip(dict1, dict2):
    keys = set(dict1.keys()).union(set(dict2.keys()))
    for k in keys:
        yield k, dict1.get(k, None), dict2.get(k, None)

def get_nested(obj, path):
    """Get nested value. Path format: '.name', '.data[0].id', '.x[1][2]'"""
    import re
    # Tokenize: split into ['.key', '[idx]', ...]
    tokens = re.findall(r'\.[^.\[]+|\[\d+\]', path)
    return _get_nested_impl(obj, tokens)

def _get_nested_impl(obj, tokens):
    if not tokens:
        return obj

    token = tokens[0]
    if token[0] == '.':
        return _get_nested_impl(obj[token[1:]], tokens[1:])
    else:  # token[0] == '['
        return _get_nested_impl(obj[int(token[1:-1])], tokens[1:])

class AttributeComparer:
    def __init__(self, identifier, casts):
        self.identifier = identifier
        self.casts = casts
        self.mismatches = []

    def cast_attr(self, key, value):
        if key not in self.casts:
            return value

        cfg = self.casts[key]
        if isinstance(cfg, str):
            if cfg == "string":
                return str(value)
            elif cfg == "json":
                return json.loads(value) if isinstance(value, str) else value
            elif cfg is None:
                return None
            else:
                raise ValueError(f"Unknown cast type: {cfg}")

        # Dict-based comparator
        comparator = cfg.get('comparator')
        if comparator == 'list_to_dict':
            if not isinstance(value, list):
                raise ValueError(f"list_to_dict requires list: {key}")
            return {get_nested(item, cfg['key']): item for item in value}

        raise ValueError(f"Unknown comparator: {comparator}")

    def compare_recurse(self, key, v1, v2):
        v1 = self.cast_attr(key, v1)
        v2 = self.cast_attr(key, v2)

        if type(v1) is not type(v2):
            # TODO: configurable tolerance for type differences
            if isinstance(v2, bool) and bool(v1) == v2:
                return
            if isinstance(v2, str) and str(v1) == v2:
                return
            self.mismatches.append((self.identifier, key, v1, v2))
            return

        if isinstance(v1, dict):
            for k in set(v1.keys()).union(set(v2.keys())):
                self.compare_recurse(f"{key}.{k}", v1.get(k), v2.get(k))
        elif isinstance(v1, list):
            if len(v1) != len(v2):
                self.mismatches.append((self.identifier, key, v1, v2))
                return
            for idx in range(len(v1)):
                self.compare_recurse(f"{key}[{idx}]", v1[idx], v2[idx])
        else:
            if v1 != v2:
                self.mismatches.append((self.identifier, key, v1, v2))

    def compare(self, attr1, attr2):
        if attr1 is None or attr2 is None:
            raise ValueError(f"Missing attributes for identifier: {self.identifier}")

        self.compare_recurse("", attr1, attr2)

        if self.mismatches:
            for gid, k, v1, v2 in self.mismatches:
                print(f"MISMATCH gml_id={gid} key={k} v1={repr(v1)} v2={repr(v2)}")
            raise ValueError(f"Attribute mismatches found for identifier: {self.identifier}")

def analyze_attributes(gml_id, attr1, attr2, casts):
    comparer = AttributeComparer(gml_id, casts)
    comparer.compare(attr1, attr2)

def test_json_attributes(fme_path, flow_path, cfg):
    """Compare JSON files between FME and Flow outputs. Config format: {name: {path: <relative_path>, casts: {...}}}"""
    for name, file_cfg in cfg.items():
        file_path = file_cfg.get('path')
        if not file_path:
            raise ValueError(f"Missing 'path' in json_attributes config for '{name}'")

        casts = file_cfg.get('casts', {})

        fme_file = fme_path / file_path
        flow_file = flow_path / file_path

        if not fme_file.exists():
            raise ValueError(f"FME JSON file not found: {fme_file}")
        if not flow_file.exists():
            raise ValueError(f"Flow JSON file not found: {flow_file}")

        # FME produces BOM UTF-8 files (unpredictably), need utf-8-sig
        with open(fme_file, 'r', encoding='utf-8-sig') as f:
            fme_data = json.load(f)
        with open(flow_file, 'r', encoding='utf-8-sig') as f:
            flow_data = json.load(f)

        log.debug(f"Comparing JSON file: {name} ({file_path})")
        analyze_attributes(name, fme_data, flow_data, casts)
