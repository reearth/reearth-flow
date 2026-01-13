#!/usr/bin/env python3
"""
Migration script to create symlinks for codelists and schemas directories.

This script creates symlinks from testcase directories to the corresponding
codelists and schemas in the artifacts/citymodel directory.
"""

import os
import re
from pathlib import Path
import tomllib


def extract_citymodel_name(citygml_zip_name: str) -> str:
    """
    Extract citymodel name from citygml_zip_name, removing feature type suffix.

    Example:
        "22203_numazu-shi_city_2023_citygml_3_op_area.zip"
        -> "22203_numazu-shi_city_2023_citygml_3_op"
    """
    # Remove .zip extension
    zip_stem = citygml_zip_name.removesuffix(".zip")

    # Remove feature type suffix if present (e.g., "_op_area", "_op_bldg")
    if "_op_" in zip_stem:
        parts = zip_stem.rsplit("_op_", 1)
        return f"{parts[0]}_op"

    return zip_stem


def find_testcases(testcases_dir: Path):
    """Find all profile.toml files in testcases directory."""
    return list(testcases_dir.rglob("profile.toml"))


def create_relative_symlink(link_path: Path, target_path: Path, dry_run: bool = True):
    """
    Create a relative symlink from link_path to target_path.

    Args:
        link_path: Path where the symlink will be created
        target_path: Path to the target directory
        dry_run: If True, only print what would be done
    """
    # Calculate relative path from link to target
    relative_target = os.path.relpath(target_path, link_path.parent)

    if dry_run:
        print(f"[DRY RUN] Would create symlink:")
        print(f"  Link: {link_path}")
        print(f"  Target (relative): {relative_target}")
        print(f"  Target (absolute): {target_path}")
        print(f"  Link exists: {link_path.exists()}")
        print(f"  Target exists: {target_path.exists()}")
        print()
    else:
        if link_path.exists():
            if link_path.is_symlink():
                print(f"Removing existing symlink: {link_path}")
                link_path.unlink()
            else:
                print(f"WARNING: {link_path} exists and is not a symlink, skipping")
                return

        print(f"Creating symlink: {link_path} -> {relative_target}")
        link_path.symlink_to(relative_target)


def main():
    script_dir = Path(__file__).parent
    testcases_dir = script_dir / "testcases"
    artifacts_base = script_dir / "artifacts" / "citymodel"

    print("=" * 80)
    print("PLATEAU Codelists/Schemas Symlink Migration Script")
    print("=" * 80)
    print(f"Testcases directory: {testcases_dir}")
    print(f"Artifacts base: {artifacts_base}")
    print()

    # Find all profile.toml files
    profile_files = find_testcases(testcases_dir)
    print(f"Found {len(profile_files)} testcase(s)")
    print()

    dry_run = True  # Change to False to actually create symlinks
    symlinks_to_create = []

    for profile_path in profile_files:
        testcase_dir = profile_path.parent
        relative_testcase = testcase_dir.relative_to(testcases_dir)

        print(f"Processing: {relative_testcase}")

        # Parse profile.toml
        try:
            with open(profile_path, "rb") as f:
                profile = tomllib.load(f)
        except Exception as e:
            print(f"  ERROR: Failed to parse profile.toml: {e}")
            print()
            continue

        citygml_zip_name = profile.get("citygml_zip_name")
        if not citygml_zip_name:
            print(f"  WARNING: No citygml_zip_name found in profile.toml")
            print()
            continue

        print(f"  citygml_zip_name: {citygml_zip_name}")

        # Extract citymodel name
        citymodel_name = extract_citymodel_name(citygml_zip_name)
        print(f"  citymodel_name: {citymodel_name}")

        # Check if codelists and schemas exist in artifacts
        codelists_source = artifacts_base / citymodel_name / "codelists"
        schemas_source = artifacts_base / citymodel_name / "schemas"

        # Create symlinks for codelists
        if codelists_source.exists():
            codelists_link = testcase_dir / "codelists"
            symlinks_to_create.append(("codelists", codelists_link, codelists_source))
            create_relative_symlink(codelists_link, codelists_source, dry_run=dry_run)
        else:
            print(f"  INFO: Codelists not found at {codelists_source}")

        # Create symlinks for schemas
        if schemas_source.exists():
            schemas_link = testcase_dir / "schemas"
            symlinks_to_create.append(("schemas", schemas_link, schemas_source))
            create_relative_symlink(schemas_link, schemas_source, dry_run=dry_run)
        else:
            print(f"  INFO: Schemas not found at {schemas_source}")

        print()

    print("=" * 80)
    print("Summary")
    print("=" * 80)
    print(f"Total symlinks to create: {len(symlinks_to_create)}")
    print()

    if dry_run:
        print("This was a DRY RUN. No changes were made.")
        print("To actually create the symlinks, edit the script and set dry_run = False")
    else:
        print("Symlinks have been created.")

    print()


if __name__ == "__main__":
    main()
