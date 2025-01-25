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

We recommend running Sherif in your CI once [all errors are fixed](#autofix). Run it by **specifying a version instead of latest**. This is useful to prevent regressions (e.g. when adding a library to a package but forgetting to update the version in other packages of the monorepo).

When using the GitHub Action, it will search for a `sherif` script in the root `package.json` and use the same arguments automatically to avoid repeating them twice. You can override this behaviour with the `args` parameter.

<details>

<summary>GitHub Actions example</summary>

```yaml
# Using the `QuiiBz/sherif` action
name: Sherif
on:
  pull_request:
jobs:
  check:
    name: Run Sherif
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: QuiiBz/sherif@v1
        # Optionally, you can specify a version and arguments to run Sherif with:
        # with:
          # version: 'v1.1.1'
          # args: '--ignore-rule root-package-manager-field'

# Using `npx` to run Sherif
name: Sherif
on:
  pull_request:
jobs:
  check:
    name: Run Sherif
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v3
        with:
          node-version: 20
      - run: npx sherif@1.1.1
```

</details>

## Autofix

Most issues can be automatically fixed by using the `--fix` (or `-f`) flag. Sherif will automatically run your package manager's `install` command (see [No-install mode](#no-install-mode) to disable this behavior) to update the lockfile. Note that autofix is disabled in CI environments (when `$CI` is set):

```bash
sherif --fix
```

### No-install mode

If you don't want Sherif to run your packager manager's `install` command after running autofix, you can use the `--no-install` flag:

```bash
sherif --fix --no-install
```

## Rules

You can ignore a specific rule by using `--ignore-rule <name>` (or `-r <name>`):

```bash
# Ignore both rules
sherif -r packages-without-package-json -r root-package-manager-field
```

You can ignore all issues in a package by using `--ignore-package <pathOrName>` (or `-p <pathOrName>`):

```bash
# Ignore all issues in the `@repo/tools` package
sherif -p @repo/tools
# Ignore all issues for packages inside `./integrations/*`
sherif -p "./integrations/*"
```

> **Note**
> Sherif doesn't have many rules for now, but will likely have more in the future (along with more features).

#### `empty-dependencies` ❌

`package.json` files should not have empty dependencies fields.

#### `multiple-dependency-versions` ❌

A given dependency should use the same version across the monorepo.

You can ignore this rule for a specific dependency and version or all versions of a dependency if it's expected in your monorepo by using `--ignore-dependency <name@version>` / `--ignore-dependency <name>` (or `-i <name@version>` / `-i <name>`):

```bash
# Ignore only the specific dependency version mismatch
sherif -i react@17.0.2 -i next@13.2.4

# Ignore all versions mismatch of dependencies that start with @next/
sherif -i @next/*

# Completely ignore all versions mismatch of these dependencies
sherif -i react -i next
```

### `unsync-similar-dependencies` ❌

Similar dependencies in a given `package.json` should use the same version. For example, if you use both `react` and `react-dom` dependencies in the same `package.json`, this rule will enforce that they use the same version.

<details>

<summary>List of detected similar dependencies</summary>

- `react`, `react-dom`
- `eslint-config-next`, `@next/eslint-plugin-next`, `@next/font` `@next/bundle-analyzer`, `@next/third-parties`, `@next/mdx`, `next`
- `eslint-config-turbo`, `eslint-plugin-turbo`, `@turbo/gen`, `turbo-ignore`, `turbo`
- `@tanstack/eslint-plugin-query`, `@tanstack/query-async-storage-persister`, `@tanstack/query-broadcast-client-experimental`, `@tanstack/query-core`, `@tanstack/query-devtools`, `@tanstack/query-persist-client-core`, `@tanstack/query-sync-storage-persister`, `@tanstack/react-query`, `@tanstack/react-query-devtools`, `@tanstack/react-query-persist-client`, `@tanstack/react-query-next-experimental`, `@tanstack/solid-query`, `@tanstack/solid-query-devtools`, `@tanstack/solid-query-persist-client`, `@tanstack/svelte-query`, `@tanstack/svelte-query-devtools`, `@tanstack/svelte-query-persist-client`, `@tanstack/vue-query`, `@tanstack/vue-query-devtools`, `@tanstack/angular-query-devtools-experimental`, `@tanstack/angular-query-experimental`

</details>

#### `non-existant-packages` ⚠️

All paths defined in the workspace (the root `package.json`' `workspaces` field or `pnpm-workspace.yaml`) should match at least one package.

#### `packages-without-package-json` ⚠️

All packages matching the workspace (the root `package.json`' `workspaces` field or `pnpm-workspace.yaml`) should have a `package.json` file.

#### `root-package-dependencies` ⚠️

The root `package.json` is private, so making a distinction between `dependencies` and `devDependencies` is useless - only use `devDependencies`.

#### `root-package-manager-field` ❌

The root `package.json` should specify the package manager and version to use. Useful for tools like corepack.

#### `root-package-private-field` ❌

The root `package.json` should be private to prevent accidentaly publishing it to a registry.

#### `types-in-dependencies` ❌

Private packages shouldn't have `@types/*` in `dependencies`, since they don't need it at runtime. Move them to `devDependencies`.

#### `unordered-dependencies` ❌

Dependencies should be ordered alphabetically to prevent complex diffs when installing a new dependency via a package manager.

## Credits

- [dedubcheck](https://github.com/innovatrics/dedubcheck) that given me the idea for Sherif
- [Manypkg](https://github.com/Thinkmill/manypkg) for some of their rules
- [This article](https://blog.orhun.dev/packaging-rust-for-npm/) for the Rust releases on NPM

## Sponsors

![Sponsors](https://github.com/QuiiBz/dotfiles/blob/main/sponsors.png?raw=true)

## License

[MIT](./LICENSE)
