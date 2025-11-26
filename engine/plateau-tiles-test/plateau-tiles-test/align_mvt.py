import sys
import mapbox_vector_tile
from pathlib import Path
from shapely.geometry import shape
from .compare_attributes import dict_zip, analyze_attributes
from . import log

def load_mvt(path):
    with open(path, "rb") as f:
        return mapbox_vector_tile.decode(f.read())

def features_by_gml_id(layer_data):
    result = {}
    for feature in layer_data['features']:
        gml_id = feature['properties'].get('gml_id')
        if gml_id:
            result[gml_id] = feature
    return result

def align_mvt_file(tile1, tile2, path=None):
    t1 = tile1 if tile1 else {}
    t2 = tile2 if tile2 else {}
    for name, l1, l2 in dict_zip(t1, t2):
        yield from align_mvt_layer(l1, l2, path)

def normalize_geometry(geom, extent):
    from shapely.affinity import scale
    return scale(geom, xfact=1/extent, yfact=1/extent, origin=(0, 0))

def align_mvt_layer(l1, l2, path=None):
    d1 = features_by_gml_id(l1) if l1 else {}
    d2 = features_by_gml_id(l2) if l2 else {}
    e1 = l1.get('extent', 4096) if l1 else 4096
    e2 = l2.get('extent', 4096) if l2 else 4096

    for gid, f1, f2 in dict_zip(d1, d2):
        g1 = normalize_geometry(shape(f1['geometry']), e1) if f1 else None
        g2 = normalize_geometry(shape(f2['geometry']), e2) if f2 else None
        yield (path, gid, g1, g2)

def z_from_path(p):
    try:
        return int(p.split('/')[1])
    except:
        return None

def align_mvt(d1, d2, zmin=None, zmax=None):
    r1 = [p.relative_to(d1) for p in d1.rglob("*.pbf")]
    r2 = [p.relative_to(d2) for p in d2.rglob("*.pbf")]

    if zmin is not None or zmax is not None:
        def filt(p):
            z = z_from_path(str(p))
            if z is None:
                return True
            if zmin is not None and z < zmin:
                return False
            if zmax is not None and z > zmax:
                return False
            return True
        r1 = [p for p in r1 if filt(str(p))]
        r2 = [p for p in r2 if filt(str(p))]

    t1 = {str(r): load_mvt(d1 / r) for r in r1}
    t2 = {str(r): load_mvt(d2 / r) for r in r2}

    for path, tile1, tile2 in dict_zip(t1, t2):
        yield from align_mvt_file(tile1, tile2, path)

def load_mvt_attr(d):
    ret = {}
    rel = {}
    for p in d.rglob("*.pbf"):
        with open(p, "rb") as f:
            tile = mapbox_vector_tile.decode(f.read())
        for k, v in tile.items():
            for feature in v["features"]:
                props = feature["properties"]
                gml_id = props["gml_id"]
                if gml_id in ret:
                    path = rel[gml_id]
                    if ret[gml_id].items() != props.items():
                        raise ValueError(f"conflicting {gml_id}: {ret[gml_id]} != {props}, file1={path}, file2={p}")
                else:
                    ret[gml_id] = props
                    rel[gml_id] = p
    return ret

def align_mvt_attr(d1, d2):
    map1 = load_mvt_attr(d1)
    map2 = load_mvt_attr(d2)
    log.info(f"Loaded MVT attributes: {len(map1)} from {d1}, {len(map2)} from {d2}")
    yield from dict_zip(map1, map2)

def test_mvt_attributes(fme_path, flow_path, cfg):
    casts = cfg.get('casts', {})

    fme_tops = {p.relative_to(fme_path).parts[0] for p in fme_path.rglob("*.pbf") if len(p.relative_to(fme_path).parts) > 1}
    flow_tops = {p.relative_to(flow_path).parts[0] for p in flow_path.rglob("*.pbf") if len(p.relative_to(flow_path).parts) > 1}

    assert fme_tops == flow_tops, f"MVT top-level directories differ: FME={fme_tops}, Flow={flow_tops}"
    for top_dir in sorted(fme_tops.union(flow_tops)):
        for gml_id, attr1, attr2 in align_mvt_attr(fme_path / top_dir, flow_path / top_dir):
            analyze_attributes(gml_id, attr1, attr2, casts)