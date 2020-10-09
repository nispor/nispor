
<!-- vim-markdown-toc GFM -->

* [Setup Test environment](#setup-test-environment)
    * [Bond](#bond)
    * [Bridge](#bridge)
    * [Bridge with VLAN filtering](#bridge-with-vlan-filtering)
    * [Clean up](#clean-up)
* [Design](#design)
    * [Rust module](#rust-module)
    * [Command line tool](#command-line-tool)
    * [Varlink service](#varlink-service)
    * [Python binding](#python-binding)
* [Release workflow](#release-workflow)

<!-- vim-markdown-toc -->

## Setup Test environment

### Bond
`./tools/test_env bond`

### Bridge

`./tools/test_env br`

### Bridge with VLAN filtering

`./tools/test_env brv`

### Clean up

`./tools/test_env rm`

## Design

### Rust module

Path: `src/lib`

### Command line tool

Path: `src/cli`

### Varlink service

Path: `src/varlink`

### Python binding

Path: `src/python`

## Release workflow

```bash
sed -i -e 's/0.6.0/0.6.1/' \
    Makefile src/*/Cargo.toml src/python/setup.py .cargo/config.toml
```

```bash
git log --oneline v0.6.0..HEAD
```
