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

Run `sherif` in the root of your monorepo to list the found issues:

```bash
# PNPM
pnpm dlx sherif@latest
# NPM
npx sherif@latest
```

Any error will cause Sherif to exit with a code 1. We recommend running Sherif in your CI once all errors are fixed. This is useful to prevent regressions (e.g. when adding a library to a package but forgetting to update the version in other packages of the monorepo)

## Rules

You can use `--ignore-rule <name>` (or `-r <name>`) to ignore one or multiple rules, and `--ignore-package <name>` (or `-p <name>`) to ignore one or multiple packages.

> **Note**
> Sherif doesn't have many rules for now, but will likely have more in the future (along with more features).

#### `empty-dependencies`

`package.json` files should not have empty dependencies fields.

#### `multiple-dependency-versions`

A given dependency should use the same version across the monorepo.

You can use `--ignore-dependency <name>` (or `-i <name>`) to ignore a dependency and allow having multiple versions of it.

#### `root-package-dependencies`

The root `package.json` is private, so making a distinction between `dependencies` and `devDependencies` is useless - only use `devDependencies`.

#### `root-package-manager-field`

The root `package.json` should specify the package manager and version to use. Useful for tools like corepack.

#### `root-package-private-field`

The root `package.json` should be private to prevent accidentaly publishing it to a registry.

## Credits

- [dedubcheck](https://github.com/innovatrics/dedubcheck) that given me the idea for Sherif
- [Manypkg](https://github.com/Thinkmill/manypkg) for some of their rules
- [This article](https://blog.orhun.dev/packaging-rust-for-npm/) for the Rust releases on NPM

## Sponsors

![Sponsors](https://github.com/QuiiBz/dotfiles/blob/main/sponsors.png?raw=true)

## License

[MIT](./LICENSE)

