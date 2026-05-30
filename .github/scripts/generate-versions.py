#!/usr/bin/env python3
"""Generate versions.json from deployed version directories.

This script scans the current directory for version-like directories
(master, stable, v0.7.5, v0.8.0-beta-1, etc.) and generates a versions.json
file that lists all available documentation versions.

The script also determines which version should be marked as 'stable'.

Usage:
    python3 generate-versions.py > versions.json
"""

import json
import os
import re
import sys


def main():
    # Find all version-like directories (v0.7.5, main, stable, v0.8.0-beta-1, etc.)
    version_pattern = re.compile(
        r'^(master|stable|v\d+\.\d+\.\d+(-[a-z0-9.-]+)?)$', re.IGNORECASE
    )
    dirs = []
    for d in os.listdir('.'):
        if os.path.isdir(d) and d != '.git' and version_pattern.match(d):
            dirs.append(d)

    dirs.sort(key=lambda x: (x != 'master', x != 'stable', x), reverse=False)

    versions = []
    stable_tag = None

    for tag in dirs:
        # Determine label
        if tag == 'master':
            label = 'Development (master)'
        elif tag == 'stable':
            label = 'Stable'
            stable_tag = tag
        else:
            label = tag

        versions.append({'tag': tag, 'label': label, 'url': f'/{tag}/'})

    # If 'stable' exists, use it; otherwise find the latest stable version (no pre-release)
    if not stable_tag:
        for tag in reversed(dirs):
            if re.match(r'^v\d+\.\d+\.\d+$', tag):  # No pre-release suffix
                stable_tag = tag
                break

    output = {'stable': stable_tag, 'versions': versions}

    print(json.dumps(output, indent=2))


if __name__ == '__main__':
    main()
