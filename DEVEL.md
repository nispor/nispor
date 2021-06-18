
<!-- vim-markdown-toc GFM -->

* [Setup Test environment](#setup-test-environment)
    * [Bond](#bond)
    * [Bridge](#bridge)
    * [Bridge with VLAN filtering](#bridge-with-vlan-filtering)
    * [Clean up](#clean-up)
* [Design](#design)
    * [Rust module](#rust-module)
    * [Command line tool](#command-line-tool)
    * [Python binding](#python-binding)
    * [Check list for creating PR:](#check-list-for-creating-pr)
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

### Python binding

Path: `src/python`

### Check list for creating PR:

 * Use `cargo fmt` to format your code. You may use
   [rust-lang/rust.vim][rust-vim]:
```vim
let g:rustfmt_autosave = 0
autocmd FileType rust nnoremap <silent> <leader>f :RustFmt<cr>
```
 * The `sudo make check` should pass.

## Release workflow

```bash
sed -i -e 's/1.1.1/1.1.2/' \
    Makefile.inc src/*/Cargo.toml src/python/setup.py .cargo/config.toml
```

```bash
git log --oneline v1.1.0..HEAD
```

[rust-vim]: https://github.com/rust-lang/rust.vim
