import shapely
from shapely.geometry import box, MultiLineString

CLIP_BOUNDS = box(0, 0, 1, 1)

def extract_lines(geom):
    if geom is None or geom.is_empty:
        return None
    lines = []
    geom_type = geom.geom_type
    if geom_type in ('Polygon', 'MultiPolygon'):
        polygons = [geom] if geom_type == 'Polygon' else list(geom.geoms)
        for poly in polygons:
            lines.append(poly.exterior)
            lines.extend(poly.interiors)
    elif geom_type in ('LineString', 'LinearRing'):
        lines.append(geom)
    elif geom_type in ('MultiLineString', 'GeometryCollection'):
        for sub_geom in geom.geoms:
            sub_lines = extract_lines(sub_geom)
            if sub_lines:
                if hasattr(sub_lines, 'geoms'):
                    lines.extend(sub_lines.geoms)
                else:
                    lines.append(sub_lines)
    if not lines:
        return None
    return MultiLineString(lines) if len(lines) > 1 else lines[0]

def clip_geometry(geom):
    if geom is None or geom.is_empty:
        return None
    clipped = geom.intersection(CLIP_BOUNDS)
    return clipped if not clipped.is_empty else None

def compare_polygons(geom1, geom2):
    if geom1 and not geom1.is_valid:
        geom1 = geom1.buffer(0)
    if geom2 and not geom2.is_valid:
        geom2 = geom2.buffer(0)
    geom1 = clip_geometry(geom1)
    geom2 = clip_geometry(geom2)
    if geom1 is None and geom2 is None:
        return ("both_missing", 0.0)
    if geom1 is None or geom2 is None:
        single = geom2 if geom1 is None else geom1
        return ("only2" if geom1 is None else "only1", single.area)
    sym_diff = geom1.symmetric_difference(geom2)
    return ("compared", sym_diff.area if not sym_diff.is_empty else 0.0)

def compare_lines(geom1, geom2):
    lines1 = clip_geometry(extract_lines(geom1))
    lines2 = clip_geometry(extract_lines(geom2))
    if lines1 is None and lines2 is None:
        return ("both_missing", 0.0)
    if lines1 is None or lines2 is None:
        single = lines2 if lines1 is None else lines1
        return ("only2" if lines1 is None else "only1", single.length)
    return ("compared", shapely.hausdorff_distance(lines1, lines2, densify=0.01))

def compare_3d_lines(geom1, geom2):
    """Compare 3D lines without clipping (for union geometries)."""
    lines1 = extract_lines(geom1)
    lines2 = extract_lines(geom2)
    if lines1 is None and lines2 is None:
        return ("both_missing", 0.0)
    if lines1 is None or lines2 is None:
        single = lines2 if lines1 is None else lines1
        return ("only2" if lines1 is None else "only1", single.length)
    return ("compared", shapely.hausdorff_distance(lines1, lines2, densify=0.01))
