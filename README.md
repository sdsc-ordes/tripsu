<p align="center">
  <img src="./docs/assets/logo.png" alt="tripsu logo" width="250">
</p>

<h1 align="center">
  tripsu
</h1>
<p align="center">
</p>
<p align="center">
  <a href="https://github.com/sdsc-ordes/tripsu/releases/latest">
    <img src="https://img.shields.io/github/release/sdsc-ordes/tripsu.svg?style=for-the-badge" alt="Current Release label" /></a>
  <a href="https://github.com/sdsc-ordes/tripsu/actions/workflows/main-and-pr.yaml">
    <img src="https://img.shields.io/github/actions/workflow/status/sdsc-ordes/tripsu/main-and-pr.yaml?label=tests&style=for-the-badge" alt="Test Status label" /></a>
  <a href="http://www.apache.org/licenses/LICENSE-2.0.html">
    <img src="https://img.shields.io/badge/LICENSE-Apache2.0-ff69b4.svg?style=for-the-badge" alt="License label" /></a>
</p>

`tripsu` (/tɹˈɪpsˈuː/, **trip**le **pseu**donymizer) is a tool to protect
sensitive values in [RDF triples](https://en.wikipedia.org/wiki/Semantic_triple)
through [pseudonymization](https://en.wikipedia.org/wiki/Pseudonymization). The
goal is to offer a fast, secure and memory-efficient pseudonymization solution
to any RDF graph.

Note: code is still in development and we support only
[NTriples format](https://en.wikipedia.org/wiki/N-Triples) as input.

The tool works in two steps:

1. Indexing to create a reference to all
   [rdf:type](https://www.w3.org/TR/rdf12-schema/#ch_type) instances in the
   graph
2. Pseudonymization to encrypt or hash sensitive parts of any RDF triple in the
   graph via a human-readable configuration file and the previously generated
   index

<details>
    <summary>Table of Content</summary>

<!--toc:start-->

- [Installation](#installation)
  - [Container](#container)
  - [Source Build](#source-build)
- [Usage](#usage)
- [Development](#development)
<!--toc:end-->

</details>

## Installation

### Container

Run the container image directly with `docker` or `podman`:

```shell
docker run -it ghcr.io/sdsc-ordes/tripsu:0.0.1 --help
```

### Source Build

The package can be compiled from source using
[cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```shell
git clone https://github.com/sdsc-ordes/tripsu
cd tripsu
cargo build --release

./target/release/tripsu --help
```

<!-- prettier-ignore -->
> [!TIP]
> Check the [development section](#development) for other setups (Nix
> etc.).

## Usage

The general command-line interface outlines the two main steps of the tool,
indexing and pseudonymization:

```shell
tripsu --help
```

which outputs

```text
A tool to pseudonymize URIs and values in RDF graphs.

Usage: tripsu <COMMAND>

Commands:
  index   1. Pass: Create a node-to-type index from input triples
  pseudo  2. Pass: Pseudonymize input triples
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Indexing only requires an RDF file as input:

```shell
tripsu index input.nt > index.nt
```

Pseudonymization requires an RDF file, index and rules configuration as input:

```shell
tripsu pseudo --index index.nt --rules rules.yaml input.nt > output.nt
```

By default, pseudonymization uses a random key. To make the process
deterministic, you may provide a file containing a fixed key with `--secret`.

In both subcommands, the input defaults to stdin and the output to stdout,
allowing to pipe both up- and downstream `tripsu` (see next section).

<!-- prettier-ignore -->
> [!TIP]
> Each subcommand supports the `--help` option to show all options. For
> more information about use-cases and configuration, see the
> [tutorial](docs/tutorial.md).

## Development

Read first the [Contribution Guidelines](/CONTRIBUTING.md).

For technical documentation on setup and development, see the
[Development Guide](docs/development-guide.md)

## Copyright

Copyright © 2023-2024 Swiss Data Science Center (SDSC),
[www.datascience.ch](http://www.datascience.ch/). All rights reserved. The SDSC
is jointly established and legally represented by the École Polytechnique
Fédérale de Lausanne (EPFL) and the Eidgenössische Technische Hochschule Zürich
(ETH Zürich). This copyright encompasses all materials, software, documentation,
and other content created and developed by the SDSC.
