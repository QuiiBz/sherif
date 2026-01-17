<p align="center">
  <picture>
    <img alt="" height="200px" src="https://github.com/QuiiBz/sherif/blob/main/assets/logo.png" />
  </picture>
  <br />
  <b>Sherif</b>: Opinionated, zero-config linter for TypeScript & JavaScript monorepos
</p>

---

![Cover](https://github.com/QuiiBz/sherif/blob/main/assets/cover.png)

## About

Sherif is an opinionated, zero-config linter for TypeScript & JavaScript monorepos. It runs fast in any monorepo and enforces rules to provide a better, standardized DX.

## Features

- ✨ **PNPM, Bun, NPM, Yarn...**: sherif works with all package managers
- 🔎 **Zero-config**: it just works and prevents regressions
- ⚡ **Fast**: doesn't need `node_modules` installed, written in 🦀 Rust

## Installation

Run `sherif` in the root of your monorepo to list the found issues. By default, any error will cause Sherif to [exit with a code 1](#exit-code):

```bash
# PNPM
pnpm dlx sherif@latest
# Bun
bunx sherif@latest
# NPM
npx sherif@latest
# Yarn
yarn dlx sherif@latest
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
          # version: 'v1.10.0'
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
      - run: npx sherif@1.10.0
```

</details>

## Autofix

Most issues can be automatically fixed by using the `--fix` (or `-f`) flag. Sherif will automatically run your package manager's `install` command (see [No-install mode](#no-install-mode) to disable this behavior) to update the lockfile. Note that autofix is disabled in CI environments (when `$CI` is set):

```bash
sherif --fix
```

### Autofixing the [`multiple-dependency-versions`](#multiple-dependency-versions-) rule

By default, running `--fix` with the `multiple-dependency-versions` rule will ask you to select which version to use for each dependency with multiple versions across the monorepo. If that doesn't work for you (e.g., you are running Sherif in a non-interactive environment), you can use the `--select` (of `-s`) flag to automatically select the highest or lowest version of every dependency:

```bash
# Autofix and select the highest version for each dependency matching the `multiple-dependency-versions` rule
sherif --fix --select highest
```

### No-install mode

If you don't want Sherif to run your packager manager's `install` command after running autofix, you can use the `--no-install` flag:

```bash
# Autofix without running the package manager's install command
sherif --fix --no-install
```

## Exit code

By default, Sherif will exit with code `1` if any error issues are found. If you only have warning issues or no issues at all, Sherif will exit with code `0`. You can change this behavior to always exit with code `1` if any issues are found, including warnings, by using the `--fail-on-warnings` option.

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

#### `unsync-similar-dependencies` ❌

Similar dependencies in a given `package.json` should use the same version. For example, if you use both `react` and `react-dom` dependencies in the same `package.json`, this rule will enforce that they use the same version.

<details>

<summary>List of detected similar dependencies</summary>

- `react`, `react-dom`
- `eslint-config-next`, `@next/eslint-plugin-next`, `@next/font` `@next/bundle-analyzer`, `@next/third-parties`, `@next/mdx`, `next`
- `@trpc/client`, `@trpc/server`, `@trpc/next`, `@trpc/react-query`
- `eslint-config-turbo`, `eslint-plugin-turbo`, `@turbo/gen`, `turbo-ignore`, `turbo`
- `sb`, `storybook`, `@storybook/codemod`, `@storybook/cli`, `@storybook/channels`, `@storybook/addon-actions`, `@storybook/addon-links`, `@storybook/react`, `@storybook/react-native`, `@storybook/components`, `@storybook/addon-backgrounds`, `@storybook/addon-viewport`, `@storybook/angular`, `@storybook/addon-a11y`, `@storybook/addon-jest`, `@storybook/client-logger`, `@storybook/node-logger`, `@storybook/core`, `@storybook/addon-storysource`, `@storybook/html`, `@storybook/core-events`, `@storybook/svelte`, `@storybook/ember`, `@storybook/addon-ondevice-backgrounds`, `@storybook/addon-ondevice-notes`, `@storybook/preact`, `@storybook/theming`, `@storybook/router`, `@storybook/addon-docs`, `@storybook/addon-ondevice-actions`, `@storybook/source-loader`, `@storybook/preset-create-react-app`, `@storybook/web-components`, `@storybook/addon-essentials`, `@storybook/server`, `@storybook/addon-toolbars`, `@storybook/addon-controls`, `@storybook/core-common`, `@storybook/builder-webpack5`, `@storybook/core-server`, `@storybook/csf-tools`, `@storybook/addon-measure`, `@storybook/addon-outline`, `@storybook/addon-ondevice-controls`, `@storybook/instrumenter`, `@storybook/addon-interactions`, `@storybook/docs-tools`, `@storybook/builder-vite`, `@storybook/telemetry`, `@storybook/core-webpack`, `@storybook/preset-html-webpack`, `@storybook/preset-preact-webpack`, `@storybook/preset-svelte-webpack`, `@storybook/preset-react-webpack`, `@storybook/html-webpack5`, `@storybook/preact-webpack5`, `@storybook/svelte-webpack5`, `@storybook/web-components-webpack5`, `@storybook/preset-server-webpack`, `@storybook/react-webpack5`, `@storybook/server-webpack5`, `@storybook/addon-highlight`, `@storybook/blocks`, `@storybook/builder-manager`, `@storybook/react-vite`, `@storybook/svelte-vite`, `@storybook/web-components-vite`, `@storybook/nextjs`, `@storybook/types`, `@storybook/manager`, `@storybook/csf-plugin`, `@storybook/preview`, `@storybook/manager-api`, `@storybook/preview-api`, `@storybook/html-vite`, `@storybook/sveltekit`, `@storybook/preact-vite`, `@storybook/addon-mdx-gfm`, `@storybook/react-dom-shim`, `create-storybook`, `@storybook/addon-onboarding`, `@storybook/react-native-theming`, `@storybook/addon-themes`, `@storybook/test`, `@storybook/react-native-ui`, `@storybook/experimental-nextjs-vite`, `@storybook/experimental-addon-test`, `@storybook/react-native-web-vite`
- `prisma`, `@prisma/client`, `@prisma/instrumentation`
- `typescript-eslint`, `@typescript-eslint/eslint-plugin`, `@typescript-eslint/parser`
- `@stylistic/eslint-plugin-js`, `@stylistic/eslint-plugin-ts`, `@stylistic/eslint-plugin-migrate`, `@stylistic/eslint-plugin`, `@stylistic/eslint-plugin-jsx`, `@stylistic/eslint-plugin-plus`
- `playwright`, `@playwright/test`

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

## Configuration

When using many CLI arguments, it might be easier to move to the configuration format. In your root `package.json`, add a `sherif` field containing the same options as the CLI, but in camelCase. Default values are shown below:

```jsonc
{
  "sherif": {
    "fix": false,
    "select": "highest", // "highest" | "lowest"
    "noInstall": false,
    "failOnWarnings": false,
    "ignoreDependency": [], // string[]
    "ignorePackage": [], // string[]
    "ignoreRule": [] // string[]
  }
}
```

When using both the configuration in the root `package.json` and CLI arguments, the CLI arguments will take precedence.

## Credits

- [dedubcheck](https://github.com/innovatrics/dedubcheck) that given me the idea for Sherif
- [Manypkg](https://github.com/Thinkmill/manypkg) for some of their rules
- [This article](https://blog.orhun.dev/packaging-rust-for-npm/) for the Rust releases on NPM

## Sponsors

![Sponsors](https://github.com/QuiiBz/dotfiles/blob/main/sponsors.png?raw=true)

## License

[MIT](./LICENSE)
