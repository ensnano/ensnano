#!/usr/bin/env python3
"""
Generate a Graphviz digraph of internal dependencies in a Cargo workspace.
"""

import pathlib
import tomllib

PREFIX = "ensnano_"


def main():
    # Collect crates
    root = pathlib.Path(".").resolve()
    root_toml = root / "Cargo.toml"
    cargo_files = [*root.glob("**/Cargo.toml"), root_toml]

    pkg_by_path = {}
    deps_by_pkg = {}

    for cargo_toml in cargo_files:
        data = tomllib.load(cargo_toml.open("rb"))

        pkg = (data.get("package") or {}).get("name")
        assert pkg is not None

        pkg_by_path[cargo_toml] = pkg
        deps = set((data.get("dependencies") or {}).keys())
        deps_by_pkg[pkg] = deps

    root_pkg = pkg_by_path[root_toml]

    internal_pkgs = {root_pkg} | {
        name for name in pkg_by_path.values() if name.startswith(PREFIX)
    }

    # Build edges
    edges = set()
    for dependent, deps in deps_by_pkg.items():
        if dependent not in internal_pkgs:
            continue
        for dep in deps:
            if dep not in internal_pkgs or dep == dependent:
                continue
            src = "main" if dep == root_pkg else dep.removeprefix(PREFIX)
            dst = "main" if dependent == root_pkg else dependent.removeprefix(PREFIX)
            edges.add((src, dst))

    # Print digraph
    print("digraph G {")
    for src, dst in sorted(edges, key=lambda e: (e[1], e[0])):
        print(f"  {src} -> {dst};")
    print("}")


if __name__ == "__main__":
    main()
