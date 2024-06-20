# RDF Protect

A simple Rust CLI tool to protect sensitive values in
[RDF triples](https://en.wikipedia.org/wiki/Semantic_triple) through
[pseudonymization](https://en.wikipedia.org/wiki/Pseudonymization).

<details>
    <summary>Table of Content</summary>
<!--toc:start-->

- [RDF Protect](#rdf-protect)
  - [Installation & Usage](#installation-usage)
    - [Usage](#usage)
  - [Development](#development)
    - [Requirements](#requirements)
    - [Nix](#nix)
    - [Formatting](#formatting)
    - [Building](#building)
    - [Testing](#testing)

<!--toc:end-->
</details>

## Installation & Usage

TODO

### Usage

TODO

## Development

### Setup

- Rust Toolchain: You need the `rust` toolchain corresponding to
  [`rust-toochain.md`](./rust-toochain.md) installed. Install Rust with
  [`rust-up`](https://rustup.rs) and any `cargo` invocations will then
  automatically respect the [toolchain](./rust-toolchain.md).

- Command runner [`just`](https://github.com/casey/just).

- The Cargo plugin [`cargo-watch`](https://crates.io/crates/cargo-watch) for
  continuous building.

- Container manager such as [`podman`](https://podman.io),
  [`docker`](https://docker.com) for some tooling (formatting etc.).

### Development Shell with `nix`

If you have the package manager
[`nix`](https://github.com/DeterminateSystems/nix-installer) installed you can
enter a development setup easily with

```shell
nix develop ./nix#default
```

or `just nix-develop` or automatically when [`direnv`](https://direnv.net) is
installed and `direnv allow` was executed inside the repository.

**Note:** Make sure to enable `flakes` and `nix-command` in
[your `nix` config](https://nixos.wiki/wiki/Flakes#Other_Distros,_without_Home-Manager)

### Formatting

To format the whole project run

```shell
just format
```

**Note:** If you use `docker`, use `just container_mgr=docker format`

### Building

To build the tool with `cargo` run

```shell
just build
```

and for continuous building (needs):

```shell
just watch
```

### Testing

To run the tests do

```shell
just test
```
