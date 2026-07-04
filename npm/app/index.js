#!/usr/bin/env node

const { spawnSync } = require('child_process')

/**
 * Detects if the system is using musl libc.
 */
function isMusl(os) {
  if (os !== 'linux') {
    return false
  }

  try {
    const report = process.report?.getReport?.()
    const header = report?.header
    const sharedObjects = report?.sharedObjects ?? header?.sharedObjects ?? []

    if (header?.glibcVersionRuntime) {
      return false
    }

    return sharedObjects.some((sharedObject) => sharedObject.includes('ld-musl'))
  } catch (e) {
    return false
  }
}

function resolveBinary(npmPackageName, extension) {
  return require.resolve(`${npmPackageName}/bin/sherif${extension}`)
}

function getNpmPackageNames(os, arch) {
  const npmPackageName = `sherif-${os}-${arch}`

  if (isMusl(os)) {
    return [`${npmPackageName}-musl`, npmPackageName]
  }

  if (os === 'linux') {
    return [npmPackageName, `${npmPackageName}-musl`]
  }

  return [npmPackageName]
}

/**
 * Returns the executable path which is located inside `node_modules`
 * The naming convention is sherif-${os}-${arch}[-musl]
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

  const npmPackageNames = getNpmPackageNames(os, arch)

  for (const npmPackageName of npmPackageNames) {
    try {
      // Since the binary will be located inside `node_modules`, we can simply call `require.resolve`
      return resolveBinary(npmPackageName, extension)
    } catch (e) {
      // Try the next compatible libc package before reporting a missing binary.
    }
  }

  throw new Error(
    `Couldn't find application binary inside node_modules for ${npmPackageNames.join(' or ')}.`
  )
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
