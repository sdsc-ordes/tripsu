# Development guide

## Setup

- Rust Toolchain: You need the `rust` toolchain corresponding to
  [`rust-toochain.md`](./rust-toochain.md) installed. Install Rust with
  [`rust-up`](https://rustup.rs) and any `cargo` invocations will then
  automatically respect the [toolchain](./rust-toolchain.md).

- Command runner [`just`](https://github.com/casey/just).

- The Cargo plugin [`cargo-watch`](https://crates.io/crates/cargo-watch) for
  continuous building.

- Container manager such as [`podman`](https://podman.io),
  [`docker`](https://docker.com) for some tooling (formatting etc.).

## Development Shell with `nix`

If you have the package manager
[`nix`](https://github.com/DeterminateSystems/nix-installer) installed you can
enter a development setup easily with

```shell
nix ./tools/nix#default
```

or `just nix-develop` or automatically when [`direnv`](https://direnv.net) is
installed and [setup for your shell](https://direnv.net/docs/hook.html) and
`direnv allow` was executed inside the repository.

**Note:** Make sure to enable `flakes` and `nix-command` in
[your `nix` config](https://nixos.wiki/wiki/Flakes#Other_Distros,_without_Home-Manager)

## Formatting

To format the whole project run

```shell
just format
```

**Note:** If you use `docker`, use `just container_mgr=docker format`

## Building

To build the tool with `cargo` run

```shell
just build
```

and for continuous building (needs):

```shell
just watch
```

## Testing

To run the tests do

```shell
just test
```

## Build the Package & Image

To build the package with Nix run:

```shell
just nix-package
```

To build the image with Nix run:

```shell
just nix-image
```

## Upload CI Images

CI is run with some container images which can be updated with:

```shell
just upload-ci-images [<version>] [<registry>]
```

where the `<version>` should be a semantic version. **Note: By default it will
upload and overwrite the current version.**

## Prepare a Release

To prepare a release you can execute:

```shell
just release <sem-version>
```

It will:

- Check that the version is semantic version and the version does not exists
  (local and remote) and it is newer then all remote version.

- Update the `Cargo.toml` and make a commit on `main`.

- Push a prepare tag `prepare-v<version>` which triggers the
  [`release.yaml`](.github/workflows/release.yaml) pipeline.

**Note: If the release pipeline fails, you can just run this same command again.
Also rerun it when you made a mistake, it will cancel the current release (works
also when `--amend`ing on the current commit)**
