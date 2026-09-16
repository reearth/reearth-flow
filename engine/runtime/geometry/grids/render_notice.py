"""Render NOTICE.md from MANIFEST.tsv.

The manifest carries the publisher, country and licence of every embedded grid,
so the attribution notice is a pure function of it and needs no network access.
Run with --check to verify the committed file instead of rewriting it.
"""

import csv
import os
import sys

MANIFEST = "MANIFEST.tsv"
NOTICE = "NOTICE.md"

PERMISSION = (
    "The Japanese grids are used with the permission of the Geospatial "
    "Information Authority of Japan recorded in the PROJ-data source "
    "distribution: 測量法に基づく国土地理院長承認(使用)R 2JHs 501."
)


def read_manifest():
    """The PROJ-data version from the header, and one dict per grid row."""
    with open(MANIFEST, newline="") as fh:
        header = fh.readline()
        rows = list(csv.DictReader(fh, delimiter="\t"))
    if not header.startswith("# PROJ-data "):
        sys.exit(f"{MANIFEST}: first line is not a '# PROJ-data <version>' header")
    return header.removeprefix("# PROJ-data ").split()[0], rows


def render(version, rows):
    """The full text of the notice."""
    total = sum(int(r["size"]) for r in rows)
    out = ["# Third-party geodetic grids", ""]
    out.append(
        f"This product embeds {len(rows)} geodetic grid files "
        f"({total / 1048576:.1f} MiB) taken from the PROJ-data {version} "
        "distribution (<https://github.com/OSGeo/PROJ-data>). Each is "
        "redistributed under the licence its publisher applied to it, "
        "reproduced below. Generated from `MANIFEST.tsv` by `render_notice.py`; "
        "do not edit by hand. See [README.md](README.md) for how the set is "
        "maintained."
    )
    out += ["", PERMISSION, ""]
    out += ["| Grid | Publisher | Country | Licence |", "|---|---|---|---|"]
    for r in sorted(rows, key=lambda r: (r["country"], r["name"])):
        publisher = (
            f"[{r['source']}]({r['source_url']})" if r["source_url"] else r["source"]
        )
        out.append(f"| `{r['name']}` | {publisher} | {r['country']} | {r['license']} |")
    return "\n".join(out) + "\n"


def main():
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    text = render(*read_manifest())
    if "--check" in sys.argv[1:]:
        current = open(NOTICE).read() if os.path.exists(NOTICE) else ""
        if current != text:
            sys.exit(
                f"{NOTICE} is not what {MANIFEST} renders to. "
                "Run `cargo make proj-grids-notice`."
            )
        print(f"{NOTICE} matches {MANIFEST}.")
        return
    with open(NOTICE, "w") as fh:
        fh.write(text)
    print(f"wrote {NOTICE} from {MANIFEST}")


main()
