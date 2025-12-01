#!/usr/bin/env python3
"""
Migrate PLATEAU citymodel directories from short names to full names.
Updates all symlinks pointing to the renamed directories.
"""

import os
import sys
from pathlib import Path
from typing import Dict, List, Tuple

# Mapping of short names to full names based on gkk directory
RENAME_MAP = {
    "08220_tsukuba-shi": "08220_tsukuba-shi_city_2023_citygml_2_op",
    "16211_imizu-shi": "16211_imizu-shi_city_2024_citygml_1_op",
    "23201_toyohashi-shi": "23201_toyohashi-shi_city_2024_citygml_1_op",
    "43206_tamana-shi": "43206_tamana-shi_city_2024_citygml_2_op",
}

BASE_DIR = Path("/Users/tsq/Projects/reearth-flow")
CITYMODEL_DIR = BASE_DIR / "engine/runtime/examples/fixture/testdata/plateau-citymodel"


def find_symlinks_to_directory(base_path: Path, target_name: str) -> List[Path]:
    """Find all symlinks pointing to directories containing target_name."""
    symlinks = []
    for root, dirs, files in os.walk(base_path):
        root_path = Path(root)
        # Check symlinks in dirs
        for d in dirs:
            item_path = root_path / d
            if item_path.is_symlink():
                target = os.readlink(item_path)
                if target_name in target:
                    symlinks.append(item_path)
    return symlinks


def get_new_symlink_target(old_target: str, old_name: str, new_name: str) -> str:
    """Replace old directory name with new name in symlink target."""
    return old_target.replace(old_name, new_name)


def migrate_directory(dry_run: bool = True):
    """
    Migrate directories and update symlinks.

    Args:
        dry_run: If True, only print what would be done without making changes.
    """
    print(f"{'='*80}")
    print(f"PLATEAU CityModel Directory Migration")
    print(f"Mode: {'DRY RUN' if dry_run else 'REAL RUN'}")
    print(f"{'='*80}\n")

    if not CITYMODEL_DIR.exists():
        print(f"ERROR: CityModel directory not found: {CITYMODEL_DIR}")
        return False

    all_operations = []

    # Step 1: Find all affected symlinks
    print("Step 1: Scanning for affected symlinks...\n")
    symlink_updates = {}

    for old_name, new_name in RENAME_MAP.items():
        old_path = CITYMODEL_DIR / old_name
        new_path = CITYMODEL_DIR / new_name

        print(f"Checking: {old_name}")

        if not old_path.exists():
            print(f"  ⚠️  WARNING: Directory does not exist: {old_path}")
            continue

        if new_path.exists():
            print(f"  ⚠️  WARNING: Target already exists: {new_path}")
            continue

        # Find all symlinks pointing to this directory
        symlinks = find_symlinks_to_directory(BASE_DIR, old_name)
        print(f"  Found {len(symlinks)} symlinks pointing to this directory")

        symlink_updates[old_name] = {
            'new_name': new_name,
            'old_path': old_path,
            'new_path': new_path,
            'symlinks': symlinks
        }

    print(f"\n{'='*80}\n")

    # Step 2: Display planned changes
    print("Step 2: Planned changes:\n")

    total_symlinks = 0
    for old_name, info in symlink_updates.items():
        print(f"Directory: {old_name}")
        print(f"  → Rename to: {info['new_name']}")
        print(f"  → Symlinks to update: {len(info['symlinks'])}")
        total_symlinks += len(info['symlinks'])

        # Show first few symlinks as examples
        if info['symlinks']:
            print(f"  → Example symlinks:")
            for symlink in info['symlinks'][:3]:
                rel_path = symlink.relative_to(BASE_DIR)
                old_target = os.readlink(symlink)
                new_target = get_new_symlink_target(old_target, old_name, info['new_name'])
                print(f"      {rel_path}")
                print(f"        Old target: {old_target}")
                print(f"        New target: {new_target}")
            if len(info['symlinks']) > 3:
                print(f"      ... and {len(info['symlinks']) - 3} more")
        print()

    print(f"Total operations:")
    print(f"  - Directories to rename: {len(symlink_updates)}")
    print(f"  - Symlinks to update: {total_symlinks}")
    print(f"\n{'='*80}\n")

    if dry_run:
        print("DRY RUN: No changes made. Run with --real to execute.")
        return True

    # Step 3: Execute changes
    print("Step 3: Executing changes...\n")

    for old_name, info in symlink_updates.items():
        print(f"Processing: {old_name}")

        # Rename directory
        try:
            print(f"  Renaming directory...")
            info['old_path'].rename(info['new_path'])
            print(f"  ✓ Renamed: {old_name} → {info['new_name']}")
        except Exception as e:
            print(f"  ✗ ERROR renaming directory: {e}")
            continue

        # Update symlinks
        failed_symlinks = []
        for symlink in info['symlinks']:
            try:
                old_target = os.readlink(symlink)
                new_target = get_new_symlink_target(old_target, old_name, info['new_name'])

                # Remove old symlink and create new one
                symlink.unlink()
                os.symlink(new_target, symlink)
            except Exception as e:
                failed_symlinks.append((symlink, str(e)))

        if failed_symlinks:
            print(f"  ✗ Failed to update {len(failed_symlinks)} symlinks:")
            for symlink, error in failed_symlinks[:5]:
                print(f"      {symlink.relative_to(BASE_DIR)}: {error}")
        else:
            print(f"  ✓ Updated {len(info['symlinks'])} symlinks")

        print()

    print(f"{'='*80}")
    print("Migration completed!")
    print(f"{'='*80}")
    return True


if __name__ == "__main__":
    dry_run = "--real" not in sys.argv

    if not dry_run:
        print("\n⚠️  WARNING: This will make real changes to the filesystem!")
        response = input("Are you sure you want to continue? (yes/no): ")
        if response.lower() != "yes":
            print("Aborted.")
            sys.exit(0)
        print()

    success = migrate_directory(dry_run=dry_run)
    sys.exit(0 if success else 1)
