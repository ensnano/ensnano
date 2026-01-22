#!/usr/bin/env python3
"""
Generate a Graphviz digraph of internal dependencies in a Cargo workspace.
"""

import pathlib
import tomllib

PREFIX = "ensnano_"


def label(pkg, root_pkg):
    return "main" if pkg == root_pkg else pkg.removeprefix(PREFIX)


def main():
    # Collect crates
    root = pathlib.Path(".").resolve()
    root_toml = root / "Cargo.toml"
    cargo_files = [*root.glob("**/Cargo.toml"), root_toml]

    pkg_by_path = {}
    deps_by_pkg = {}

    for cargo_toml in cargo_files:
        data = tomllib.load(cargo_toml.open("rb"))
        pkg = data.get("package", {})["name"]
        pkg_by_path[cargo_toml] = pkg
        deps_by_pkg[pkg] = set(data.get("dependencies", {}).keys())

    root_pkg = pkg_by_path[root_toml]
    internal_pkgs = {root_pkg} | {
        name for name in pkg_by_path.values() if name.startswith(PREFIX)
    }

    # Build edges
    edges = set()
    for dependent, dependencies in deps_by_pkg.items():
        for dependency in dependencies:
            if dependency == dependent or dependency not in internal_pkgs:
                continue
            edges.add((label(dependent, root_pkg), label(dependency, root_pkg)))

    # Print digraph
    print("digraph G {")
    for dst, src in sorted(edges):
        print(f"  {src} -> {dst};")
    print("}")


if __name__ == "__main__":
    main()
