#!/usr/bin/env node

const { spawnSync } = require('child_process')
const fs = require('fs')

/**
 * Detects if the system is using musl libc (e.g., Alpine Linux)
 */
function isMusl() {
  if (process.platform !== 'linux') {
    return false
  }

  // Check for musl dynamic linker
  try {
    return fs.existsSync('/lib/ld-musl-x86_64.so.1') ||
           fs.existsSync('/lib/ld-musl-aarch64.so.1')
  } catch (e) {
    return false
  }
}

/**
 * Returns the executable path which is located inside `node_modules`
 * The naming convention is app-${os}-${arch}[-musl]
 * If the platform is `win32` or `cygwin`, executable will include a `.exe` extension.
 * @see https://nodejs.org/api/os.html#osarch
 * @see https://nodejs.org/api/os.html#osplatform
 * @example "x/xx/node_modules/app-darwin-arm64"
 */
function getExePath() {
  const arch = process.arch
  let os = process.platform
  let extension = ""
  if (['win32', 'cygwin'].includes(process.platform)) {
    os = 'windows'
    extension = '.exe'
  }

  if (os === 'linux') {
    const variants = isMusl()
      ? ['-musl', '']  // On musl: try musl-specific, then fall back to glibc
      : ['', '-musl']  // On glibc: try glibc (no suffix), then fall back to musl

    for (const variant of variants) {
      try {
        return require.resolve(`sherif-${os}-${arch}${variant}/bin/sherif${extension}`)
      } catch (e) {
        // Try next variant
      }
    }
    throw new Error(
      `Couldn't find application binary inside node_modules for ${os}-${arch}`
    )
  }

  try {
    // Since the binary will be located inside `node_modules`, we can simply call `require.resolve`
    return require.resolve(`sherif-${os}-${arch}/bin/sherif${extension}`)
  } catch (e) {
    throw new Error(
      `Couldn't find application binary inside node_modules for ${os}-${arch}`
    )
  }
}

/**
 * Runs the application with args using nodejs spawn
 */
function run() {
  const args = process.argv.slice(2)
  const processResult = spawnSync(getExePath(), args, { stdio: 'inherit' })

  if (processResult.error) {
    console.error(`Failed to execute sherif: ${processResult.error.message}`)
    console.error("Please report this issue: https://github.com/QuiiBz/sherif/issues")
    process.exit(1)
  }

  process.exit(processResult.status ?? 0)
}

run()
