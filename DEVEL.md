
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

## Run the tests

### Locally

`make check`

Warning: this will make changes to your system's network configuration, so this
is not recommended.

### In a container

`make check_in_container`

### Manual testing and experimenting in a container

`./tools/nispor-in-container` (see help with `--help`)

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

Please install [cargo-vendor-filterer][cargo_vendor_fitler_url],
setup [hub][hub_url] and cargo credentials, then run:

```bash
make upstream_release
```

[rust-vim]: https://github.com/rust-lang/rust.vim
[hub_url]: https://hub.github.com/
[cargo_vendor_fitler_url]: https://github.com/coreos/cargo-vendor-filterer
