import zipfile
import re
from . import log

class FilterNode:
    def __init__(self, inclusive):
        self.inclusive = inclusive
        self.children = {}

    def should_include_child(self, child_name):
        if child_name in self.children:
            return True
        return not self.inclusive

    def get_child_node(self, child_name):
        return self.children.get(child_name)

    def process(self, item):
        return item

class GmlFilterNode(FilterNode):
    def process(self, content):
        text = content.decode('utf-8')
        lines = text.splitlines(keepends=True)
        filtered_lines = []
        inside_member = False
        member_lines = []
        should_keep_member = False

        for line in lines:
            if '<core:cityObjectMember>' in line or '<cityObjectMember>' in line:
                inside_member = True
                member_lines = [line]
                should_keep_member = False
                continue

            if '</core:cityObjectMember>' in line or '</cityObjectMember>' in line:
                member_lines.append(line)
                if should_keep_member:
                    filtered_lines.extend(member_lines)
                inside_member = False
                member_lines = []
                should_keep_member = False
                continue

            if inside_member:
                member_lines.append(line)
                if not should_keep_member and 'gml:id="' in line:
                    match = re.search(r'gml:id="([^"]+)"', line)
                    if match:
                        member_id = match.group(1)
                        if self.should_include_child(member_id):
                            should_keep_member = True
            else:
                filtered_lines.append(line)

        return ''.join(filtered_lines).encode('utf-8')

def build_filter_tree(tree):
    root = FilterNode(True)

    for key, value in tree.items():
        if not key.startswith(('+', '-')):
            continue

        inclusive = key[0] == '+'
        path = key[1:]

        node = root
        if path:
            for part in path.split('/'):
                if part not in node.children:
                    node.children[part] = FilterNode(node.inclusive)
                node = node.children[part]

        node.inclusive = inclusive
        if isinstance(value, list):
            for child in value:
                if child not in node.children:
                    child_node = FilterNode(False)
                    node.children[child] = child_node

    return root

def filter_zip(src_zip, dst_zip, tree):
    root = build_filter_tree(tree)

    with zipfile.ZipFile(src_zip, 'r') as src, zipfile.ZipFile(dst_zip, 'w', zipfile.ZIP_DEFLATED) as dst:
        for item in src.infolist():
            if item.is_dir():
                continue

            parts = item.filename.split('/')
            node = root
            include = True

            for part in parts:
                if not node.should_include_child(part):
                    include = False
                    break
                child = node.get_child_node(part)
                node = child if child else node
            if include:
                log.debug(f"Including: {item.filename}")
                content = src.read(item)
                filtered_content = node.process(content)
                dst.writestr(item, filtered_content)
