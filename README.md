<p align="center">
  <picture>
    <img alt="" height="200px" src="https://github.com/QuiiBz/sherif/blob/main/assets/logo.png" />
  </picture>
  <br />
  <b>Sherif</b>: Opinionated, zero-config linter for JavaScript monorepos
</p>

---

![Cover](https://github.com/QuiiBz/sherif/blob/main/assets/cover.png)

## About

Sherif is an opinionated, zero-config linter for JavaScript monorepos. It runs fast in any monorepo and enforces rules to provide a better, standardized DX.

## Features

- ✨ **PNPM, NPM, Yarn...**: sherif works with all package managers
- 🔎 **Zero-config**: it just works and prevents regressions
- ⚡ **Fast**: doesn't need `node_modules` installed, written in 🦀 Rust

## Installation

Run `sherif` in the root of your monorepo to list the found issues. Any error will cause Sherif to exit with a code 1:

```bash
# PNPM
pnpm dlx sherif@latest
# NPM
npx sherif@latest
```

We recommend running Sherif in your CI once all errors are fixed. Run it by **specifying a version instead of latest**. This is useful to prevent regressions (e.g. when adding a library to a package but forgetting to update the version in other packages of the monorepo).

<details>

<summary>GitHub Actions example</summary>

```yaml
name: Sherif
on:
  pull_request:
jobs:
  check:
    name: Run Sherif
    runs-on: ubuntu-22.04
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - uses: actions/setup-node@v3
        with:
          node-version: 20
      - run: npx sherif@0.2.0
```

</details>

## Rules

You can ignore a specific rule by using `--ignore-rule <name>` (or `-r <name>`):

```bash
# Ignore both rules
sherif -r packages-without-package-json -r root-package-manager-field
```

You can ignore all issues in a package by using `--ignore-package <name>` (or `-p <name>`):

```bash
# Ignore all issues in the package
sherif -p @repo/tools
```

> **Note**
> Sherif doesn't have many rules for now, but will likely have more in the future (along with more features).

#### `empty-dependencies` ❌

`package.json` files should not have empty dependencies fields.

#### `multiple-dependency-versions` ❌

A given dependency should use the same version across the monorepo.

You can ignore this rule for a dependency if you expect to have multiple versions by using `--ignore-dependency <name>` (or `-i <name>`):

```bash
# Ignore dependencies that are expected to have multiple versions
sherif -i react -i @types/node
```

#### `packages-without-package-json` ⚠️

All packages defined in the root `package.json`' `workspaces` field or `pnpm-workspace.yaml` should have a `package.json` file.

#### `root-package-dependencies` ⚠️

The root `package.json` is private, so making a distinction between `dependencies` and `devDependencies` is useless - only use `devDependencies`.

#### `root-package-manager-field` ❌

The root `package.json` should specify the package manager and version to use. Useful for tools like corepack.

#### `root-package-private-field` ❌

The root `package.json` should be private to prevent accidentaly publishing it to a registry.

## Credits

- [dedubcheck](https://github.com/innovatrics/dedubcheck) that given me the idea for Sherif
- [Manypkg](https://github.com/Thinkmill/manypkg) for some of their rules
- [This article](https://blog.orhun.dev/packaging-rust-for-npm/) for the Rust releases on NPM

## Sponsors

![Sponsors](https://github.com/QuiiBz/dotfiles/blob/main/sponsors.png?raw=true)

## License

[MIT](./LICENSE)

