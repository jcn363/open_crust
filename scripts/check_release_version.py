#!/usr/bin/env /usr/bin/python3
"""
Check release version matches version.txt

Following DeepSpeed's version management pattern.
This script validates that the proposed release version matches the current version in version.txt.
"""

import sys
import subprocess
from pathlib import Path
from packaging.version import parse as parse_version


def read_version_file():
    """Read version from version.txt"""
    version_file = Path(__file__).parent.parent / "version.txt"
    if not version_file.exists():
        print(f"ERROR: version.txt not found at {version_file}")
        sys.exit(1)
    
    with open(version_file, "r") as f:
        version = f.read().strip()
    
    if not version:
        print("ERROR: version.txt is empty")
        sys.exit(1)
    
    return version


def validate_version_format(version):
    """Validate version format (no 'v' prefix, valid semver)"""
    if version.startswith('v'):
        print(f"ERROR: Version must not have 'v' prefix: {version}")
        print("       Use bare version like '1.2.0' not 'v1.2.0'")
        return False
    
    try:
        parse_version(version)
    except Exception as e:
        print(f"ERROR: Invalid version format: {version}")
        print(f"       {e}")
        return False
    
    return True


def get_cargo_version():
    """Get version from Cargo.toml"""
    cargo_file = Path(__file__).parent.parent / "Cargo.toml"
    if not cargo_file.exists():
        print(f"ERROR: Cargo.toml not found at {cargo_file}")
        sys.exit(1)
    
    with open(cargo_file, "r") as f:
        for line in f:
            if line.strip().startswith("version"):
                # Extract version from line like: version = "1.2.0"
                parts = line.split('"')
                if len(parts) >= 2:
                    return parts[1]
    
    print("ERROR: Could not find version in Cargo.toml")
    sys.exit(1)


def main():
    if len(sys.argv) != 2:
        print("Usage: python check_release_version.py <proposed_version>")
        print("Example: python check_release_version.py 1.2.0")
        sys.exit(1)
    
    proposed_version = sys.argv[1]
    current_version = read_version_file()
    cargo_version = get_cargo_version()
    
    print(f"Proposed version: {proposed_version}")
    print(f"Current version (version.txt): {current_version}")
    print(f"Cargo.toml version: {cargo_version}")
    
    # Validate format
    if not validate_version_format(proposed_version):
        sys.exit(1)
    
    if not validate_version_format(current_version):
        sys.exit(1)
    
    # Check if proposed matches current
    if proposed_version != current_version:
        print(f"ERROR: Proposed version ({proposed_version}) does not match version.txt ({current_version})")
        sys.exit(1)
    
    # Check if Cargo.toml matches
    if proposed_version != cargo_version:
        print(f"ERROR: Proposed version ({proposed_version}) does not match Cargo.toml ({cargo_version})")
        sys.exit(1)
    
    print("SUCCESS: Version validation passed")
    sys.exit(0)


if __name__ == "__main__":
    main()