import json, struct
import trimesh
import numpy as np
from shapely.geometry import Polygon

def extract_strings(binary_blob, buffer_views, values_idx, offsets_idx, count):
    # Get offsets from the OFFSETS buffer
    offsets_view = buffer_views[offsets_idx]
    offsets_data = binary_blob[offsets_view['byteOffset']: offsets_view['byteOffset'] + offsets_view['byteLength']]
    offsets = np.frombuffer(offsets_data, dtype=np.uint32)
    
    # Get string data from the VALUES buffer
    values_view = buffer_views[values_idx]
    strings_data = binary_blob[values_view['byteOffset']: values_view['byteOffset'] + values_view['byteLength']]
    
    # Extract strings
    strings = []
    assert len(offsets) - 1 == count, f"Offsets length mismatch: {len(offsets)-1} vs {count}"
    for i in range(count):
        start = offsets[i]
        end = offsets[i + 1]
        strings.append(strings_data[start:end].decode('utf-8'))
    return strings

def read_glb_json(filepath):
    with open(filepath, 'rb') as f:
        assert f.read(4) == b'glTF', "Not a valid GLB file"
        version = struct.unpack('<I', f.read(4))[0]
        assert version == 2, "Only GLB version 2 supported"
        _length = struct.unpack('<I', f.read(4))[0]
        json_chunk_length = struct.unpack('<I', f.read(4))[0]
        assert f.read(4) == b'JSON', "Expected JSON chunk"
        json_data = f.read(json_chunk_length).decode('utf-8')
        buffer_length = struct.unpack('<I', f.read(4))[0]
        assert f.read(4) == b'BIN\x00', "Expected BIN chunk"
        buffer = f.read(buffer_length)
        return json.loads(json_data), buffer

def read_glb_tile(glb_path, apply_rtc=True):
    result = {}
    # metadata extraction
    j, buf = read_glb_json(glb_path)
    props = j["extensions"]["EXT_structural_metadata"]["propertyTables"]
    assert len(props) == 1, "Only one property table supported"
    prop = props[0]
    count = prop["count"]
    for k, v in prop["properties"].items():
        offsets = v["stringOffsets"]
        values = v["values"]
        s = extract_strings(buf, j["bufferViews"], values, offsets, count)
        prop["properties"][k] = s
    # unzipping properties
    result = {i: ({k: v[i] for k, v in prop["properties"].items()}, []) for i in range(count)}

    # Extract RTC_CENTER (Relative To Center) from node translation
    rtc_center = None
    if apply_rtc and "nodes" in j and len(j["nodes"]) > 0:
        node = j["nodes"][0]
        if "translation" in node:
            rtc_center = np.array(node["translation"])
            # print(f"RTC_CENTER (ECEF): {rtc_center}")

    # geometry extraction
    scene = trimesh.load(str(glb_path))
    for geom_name, mesh in scene.geometry.items():
        assert "_FEATURE_ID_1" not in mesh.vertex_attributes, "Multiple feature IDs per mesh not supported"
        batch_ids = mesh.vertex_attributes['_FEATURE_ID_0']
        assert len(batch_ids) == len(mesh.vertices), f"{len(batch_ids)} vs {len(mesh.vertices)}"

        # Apply RTC transformation if available
        vertices = mesh.vertices.copy()
        if rtc_center is not None:
            vertices = vertices + rtc_center

        for idx, face in enumerate(mesh.faces):
            face_vertices = vertices[face]
            poly = Polygon(face_vertices)
            for vid in face:
                for batch_id in batch_ids[vid]:
                    batch_id = int(batch_id)
                    result[batch_id][1].append(poly)
    return result

if __name__ == '__main__':
    import sys
    if len(sys.argv) < 2:
        print("Usage: python tile_reader.py <tile_file>")
        sys.exit(1)
    features = read_glb_tile(sys.argv[1])
    for i, (props, geoms) in features.items():
        print(f"Feature {i}, Geometries: {len(geoms)} triangles")
        if 'gml_id' in props:
            print(f"  gml_id: {props['gml_id']}")
