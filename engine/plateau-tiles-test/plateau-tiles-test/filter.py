import zipfile
import re
from . import log

def filter_gml_content(content, gml_ids):
    """Filter GML content to only include specific gml:id members."""
    text = content.decode('utf-8')
    lines = text.splitlines(keepends=True)
    filtered_lines = []
    inside_member = False
    member_lines = []

    for line in lines:
        if '<core:cityObjectMember>' in line or '<cityObjectMember>' in line:
            inside_member = True
            member_lines = [line]
            continue

        if '</core:cityObjectMember>' in line or '</cityObjectMember>' in line:
            member_lines.append(line)
            if any(f'gml:id="{gml_id}"' in ''.join(member_lines) for gml_id in gml_ids):
                filtered_lines.extend(member_lines)
            inside_member = False
            member_lines = []
            continue

        if inside_member:
            member_lines.append(line)
        else:
            filtered_lines.append(line)

    return ''.join(filtered_lines).encode('utf-8')

def should_include_path(path, tree):
    """Check if a path should be included based on tree structure."""
    for prefix, items in tree.items():
        if not isinstance(items, list) or not all(isinstance(item, str) for item in items):
            continue

        if not path.startswith(prefix):
            continue

        rest = path[len(prefix):]
        for item in items:
            if rest.startswith(item):
                return True

    return False

# example tree which includes codelists and schemas directory and specific gml ids:
# "udx/squr/533912_squr_6697_op.gml": [gml_id1, gml_id2, ...]
# "": ["codelists/", "schemas/"]
def filter_zip(src_zip, dst_zip, tree):
    """Filter a zip file based on tree structure.

    Args:
        src_zip: Path to source zip file
        dst_zip: Path to destination zip file
        tree: Dict mapping file paths to either:
              - List of gml_ids (for GML files to filter)
              - List of directory/file names to include entirely (when key is "")
    """
    # Get directories/files to include entirely from root
    include_paths = tree.get("", [])

    with zipfile.ZipFile(src_zip, 'r') as src, zipfile.ZipFile(dst_zip, 'w', zipfile.ZIP_DEFLATED) as dst:
        for item in src.infolist():
            path = item.filename

            # Check if this is a GML file with specific gml_ids to filter (highest priority)
            if path in tree and isinstance(tree[path], list):
                gml_ids = tree[path]
                log.debug(f"Filtering GML file: {path} for {len(gml_ids)} IDs")
                content = src.read(path)
                filtered_content = filter_gml_content(content, gml_ids)
                dst.writestr(item, filtered_content)
                continue

            # Check if this path matches any tree entry
            if should_include_path(path, tree):
                log.debug(f"Including file by prefix match: {path}")
                dst.writestr(item, src.read(path))