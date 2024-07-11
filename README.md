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

## Installation & Usage

The package must be compiled from source using
[cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```shell
git clone https://github.com/sdsc-ordes/tripsu
cd tripsu
cargo build --release
# executable binary located in ./target/release/tripsu
```

### Usage

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

> [!TIP] For each subcommand, you can use `--help` to see all options.

In both subcommands, the input defaults to stdin and the output to stdout,
allowing to pipe both up- and downstream `tripsu` (see next section).

### Use Case

The main idea behind `tripsu` is to integrate smoothly into other CLI tools up-
and downstream via piping. Let us assume that we're running a SPARQL query on a
large graph and we would like to pseudonymize some of the triples. This is how
the flow should look like:

```shell
curl <sparql-query> | tripsu pseudo -i index.nt -c config.yaml > pseudo.nt
```

For this flow to stream data instead of loading everything into memory, we had
to include an indexing step to make the streaming process consistent and easier
to control. It is not as clean as having one command doing everything, but it
simplifies code development.

### Example

There are three possible ways to pseudonymize RDF triples:

1. Pseudonymize the URI of nodes with `rdf:type`.
2. Pseudonymize values for specific subject-predicate combinations.
3. Pseudonymize any value for a given predicate.

By using all three ways together, we're able to get an RDF file with sensitive
information:

<details>
    <summary><b>Click to show input</b></summary>

```ntriples
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://xmlns.com/foaf/0.1/holdsAccount> <http://example.org/Alice-Bank-Account> .
<http://example.org/Alice-Bank-Account> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/OnlineAccount> .
<http://example.org/Alice-Bank-Account> <http://schema.org/name> "my_account32" .
<http://example.org/Alice-Bank-Account> <http://schema.org/accessCode> "secret-123" .
<http://example.org/Alice> <http://schema.org/name> "Alice" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

</details>

And pseudonymize the sensitive information such as people's names, personal and
secret information while keeping the rest as is:

<details>
    <summary><b>Click to show output</b></summary>

```
<http://example.org/af321bbc> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/af321bbc> <http://xmlns.com/foaf/0.1/holdsAccount> <http://example.org/bs2313bc> .
<http://example.org/bs2313bc> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/OnlineAccount> .
<http://example.org/bs2313bc> <http://schema.org/name> "pp54r32" .
<http://example.org/bs2313bc> <http://schema.org/accessCode> "asfnd223" .
<http://example.org/af321bbc> <http://schema.org/name> "af321bbc" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

</details>

The next subsections break down each of the three pseudonymization approaches to
better understand how they operate.

#### 1. Pseudonymize the URI of nodes with `rdf:type`

<details>
    <summary><b>Click to show</b></summary>

Given the following config:

```yaml
replace_uri_of_nodes_with_type:
  - "http://xmlns.com/foaf/0.1/Person"
```

The goal is to pseudonymize all instaces of `rdf:type` Person. The following
input file:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
```

Would become:

```
<http://example.org/af321bbc> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
```

</details>

#### 2. Pseudonymize values for specific subject-predicate combinations

<details>
    <summary><b>Click to show</b></summary>

Given the following config:

```yaml
replace_values_of_subject_predicate:
  "http://xmlns.com/foaf/0.1/Person":
    - "http://schema.org/name"
```

The goal is to pseudonymize only the instances of names when they're associated
to Person. The following input file:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "Alice" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

Would become:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "af321bbc" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

</details>

#### 3. Pseudonymize any value for a given predicate

<details>
    <summary><b>Click to show</b></summary>

Given the following config:

```yaml
replace_value_of_predicate:
  - "http://schema.org/name"
```

The goal is to pseudonymize any values associated to name. The following input
file:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "Alice" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

Would become:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "af321bbc" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "38a3dd71" .
```

</details>

## Development

Read first the [Contribution Guidelines](/CONTRIBUTING.md).

For technical documentation on setup and development, see the [Development Guide](docs/development_guide.md)
