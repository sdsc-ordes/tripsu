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
  <a href="https://github.com/sdsc-ordes/tripsu/actions/workflows/normal.yaml">
    <img src="https://img.shields.io/github/actions/workflow/status/sdsc-ordes/tripsu/normal.yaml?label=tests&style=for-the-badge" alt="Test Status label" /></a>
  <a href="http://www.apache.org/licenses/LICENSE-2.0.html">
    <img src="https://img.shields.io/badge/LICENSE-Apache2.0-ff69b4.svg?style=for-the-badge" alt="License label" /></a>
</p>

tripsu (/tɹˈɪpsˈuː/, **trip**le **pseu**donymizer) is a tool to protect
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

- [tripsu](#tripsu)
  - [Installation & Usage](#installation-usage)
    - [Usage](#usage)
    - [Use Case](#use-case)
    - [Example](#example)
      - [1. Pseudonymize the URI of nodes with `rdf:type`](#1-pseudonymize-the-uri-of-nodes-with-rdftype)
      - [2. Pseudonymize values for specific subject-predicate combinations](#2-pseudonymize-values-for-specific-subject-predicate-combinations)
      - [3. Pseudonymize any value for a given predicate](#3-pseudonymize-any-value-for-a-given-predicate)
  - [Development](#development) - [Setup](#setup) -
  [Development Shell with `nix`](#development-shell-with-nix) -
  [Formatting](#formatting) - [Building](#building) - [Testing](#testing)
  <!--toc:end-->

</details>

## Installation

The package must be compiled from source using
[cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```shell
git clone https://github.com/sdsc-ordes/tripsu
cd tripsu
cargo build --release
# executable binary located in ./target/release/tripsu
```

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

Pseudonomyzation requires an RDF file, index and config as input:

```shell
tripsu pseudo --index index.nt --config rules.yaml input.nt > output.nt
```

> [!TIP]
> For each subcommand, you can use `--help` to see all options.

In both subcommands, the input defaults to stdin and the output to stdout,
allowing to pipe both up- and downstream `tripsu` (see next section).

For more information about use-cases and configuration, see the [tutorial](docs/tutorial.md).

## Development

Read first the [Contribution Guidelines](/CONTRIBUTING.md).

For technical documentation on setup and development, see the [Development Guide](docs/development_guide.md)
