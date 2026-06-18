#!/usr/bin/env /usr/bin/python3
"""
Bump patch version in version.txt

Following DeepSpeed's version management pattern.
This script increments the patch version after a successful release.
"""

import sys
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


def write_version_file(version):
    """Write version to version.txt"""
    version_file = Path(__file__).parent.parent / "version.txt"
    with open(version_file, "w") as f:
        f.write(version + "\n")


def bump_patch_version(version):
    """Bump patch version using semantic versioning"""
    try:
        parsed = parse_version(version)
    except Exception as e:
        print(f"ERROR: Invalid version format: {version}")
        print(f"       {e}")
        sys.exit(1)
    
    # Get version components
    major = parsed.major
    minor = parsed.minor
    micro = parsed.micro
    
    # Increment patch (micro) version
    new_micro = micro + 1
    new_version = f"{major}.{minor}.{new_micro}"
    
    return new_version


def main():
    current_version = read_version_file()
    print(f"Current version: {current_version}")
    
    new_version = bump_patch_version(current_version)
    print(f"New version: {new_version}")
    
    write_version_file(new_version)
    print(f"Updated version.txt to {new_version}")
    
    # Also update Cargo.toml
    cargo_file = Path(__file__).parent.parent / "Cargo.toml"
    if cargo_file.exists():
        with open(cargo_file, "r") as f:
            content = f.read()
        
        # Replace version in Cargo.toml
        import re
        new_content = re.sub(
            r'version = "[^"]+"',
            f'version = "{new_version}"',
            content
        )
        
        with open(cargo_file, "w") as f:
            f.write(new_content)
        
        print(f"Updated Cargo.toml to {new_version}")
    
    print("SUCCESS: Version bumped")
    sys.exit(0)


if __name__ == "__main__":
    main()