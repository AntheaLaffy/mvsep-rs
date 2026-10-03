#!/usr/bin/env python3
"""Check or update the independent desktop and API crate release versions."""
import argparse
import json
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
SEMVER = r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)'


def version_tuple(value):
    if not re.fullmatch(SEMVER, value):
        raise ValueError(f'Expected a stable MAJOR.MINOR.PATCH version, got {value!r}')
    return tuple(map(int, value.split('.')))


def read_toml(path):
    return tomllib.loads((ROOT / path).read_text())


def lock_version(path, name):
    entries = [p for p in read_toml(path)['package'] if p['name'] == name]
    if len(entries) != 1:
        raise ValueError(f'{path}: expected one package named {name}')
    return entries[0]['version']


def check(tag=None):
    desktop = read_toml('src-tauri/Cargo.toml')['package']['version']
    api = read_toml('test-api/Cargo.toml')['package']['version']
    package = json.loads((ROOT / 'package.json').read_text())
    npm_lock = json.loads((ROOT / 'package-lock.json').read_text())
    values = {
        'package.json': package['version'],
        'package-lock.json': npm_lock['version'],
        'package-lock.json root package': npm_lock['packages']['']['version'],
        'src-tauri/tauri.conf.json': json.loads((ROOT / 'src-tauri/tauri.conf.json').read_text())['version'],
        'src-tauri/Cargo.lock': lock_version('src-tauri/Cargo.lock', 'mvsep-gui'),
    }
    for name, value in values.items():
        if value != desktop:
            raise ValueError(f'{name}: {value} differs from desktop version {desktop}')
    if lock_version('test-api/Cargo.lock', 'mvsep-api-tester') != api:
        raise ValueError('test-api/Cargo.lock differs from API crate version')
    version_tuple(desktop)
    version_tuple(api)
    if tag and tag != f'v{desktop}':
        raise ValueError(f'Release tag {tag!r} differs from v{desktop}')
    return desktop, api


def replace_package_version(text, name, value, lock=False):
    header = r'\[\[package\]\]' if lock else r'\[package\]'
    pattern = rf'({header}\nname = "{re.escape(name)}"\nversion = ")[^"]+(")'
    # Match just the selected package, never transitive versions with the same number.
    result, count = re.subn(pattern, lambda m: m[1] + value + m[2], text)
    if count != 1:
        raise ValueError(f'Expected one version field for {name}, found {count}')
    return result


def bump(target, value):
    desktop, api = check()
    current = desktop if target == 'desktop' else api
    if version_tuple(value) <= version_tuple(current):
        raise ValueError(f'New version {value} must be greater than {current}')
    updates = {}
    if target == 'desktop':
        for path in ['package.json', 'package-lock.json', 'src-tauri/tauri.conf.json']:
            data = json.loads((ROOT / path).read_text())
            data['version'] = value
            if path == 'package-lock.json':
                data['packages']['']['version'] = value
            updates[path] = json.dumps(data, indent=2, ensure_ascii=False) + '\n'
        crate, name = 'src-tauri', 'mvsep-gui'
    else:
        crate, name = 'test-api', 'mvsep-api-tester'
    for filename in ['Cargo.toml', 'Cargo.lock']:
        path = f'{crate}/{filename}'
        updates[path] = replace_package_version((ROOT / path).read_text(), name, value, filename.endswith('.lock'))
    for path, content in updates.items():
        (ROOT / path).write_text(content)
    check()
    print(f'{target}: {current} -> {value}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    checker = sub.add_parser('check')
    checker.add_argument('--tag', help='Require a desktop release tag such as v1.2.3')
    updater = sub.add_parser('bump')
    updater.add_argument('target', choices=['desktop', 'api'])
    updater.add_argument('version')
    args = parser.parse_args()
    try:
        if args.command == 'bump':
            bump(args.target, args.version)
        else:
            desktop, api = check(args.tag)
            print(f'Versions consistent: desktop {desktop}; API crate {api}')
    except (ValueError, KeyError) as error:
        print(f'Version check failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
