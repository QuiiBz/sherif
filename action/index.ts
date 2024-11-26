import * as core from '@actions/core';
import * as tc from '@actions/tool-cache';
import * as github from '@actions/github';
import * as exec from '@actions/exec';
import * as os from 'os';
import * as path from 'path';
import * as fsp from 'fs/promises';

async function run(): Promise<void> {
  try {
    // Get inputs
    const version = core.getInput('version');
    const token = core.getInput('github-token');
    let additionalArgs = core.getInput('args');

    // Initialize octokit
    const octokit = github.getOctokit(token);

    // Determine release to download
    let releaseTag = version;
    if (version === 'latest') {
      const latestRelease = await octokit.rest.repos.getLatestRelease({
        owner: 'quiibz',
        repo: 'sherif'
      });
      releaseTag = latestRelease.data.tag_name;
    }

    // Get platform and architecture specific details
    const platform = os.platform();
    const arch = os.arch();

    // Map platform and architecture to release asset names
    const platformTargets: Record<string, Record<string, string>> = {
      'darwin': {
        'arm64': 'aarch64-apple-darwin',
        'x64': 'x86_64-apple-darwin'
      },
      'win32': {
        'arm64': 'aarch64-pc-windows-msvc',
        'x64': 'x86_64-pc-windows-msvc'
      },
      'linux': {
        'arm64': 'aarch64-unknown-linux-gnu',
        'x64': 'x86_64-unknown-linux-gnu'
      }
    };

    const platformTarget = platformTargets[platform]?.[arch];
    if (!platformTarget) {
      throw new Error(`Unsupported platform (${platform}) or architecture (${arch})`);
    }

    // Construct asset name
    const assetName = `sherif-${platformTarget}.zip`;

    // Get release assets
    const release = await octokit.rest.repos.getReleaseByTag({
      owner: 'quiibz',
      repo: 'sherif',
      tag: releaseTag
    });

    const asset = release.data.assets.find(a => a.name === assetName);
    if (!asset) {
      throw new Error(`Could not find asset ${assetName} in release ${releaseTag}`);
    }

    // Download the zip file
    core.info(`Downloading Sherif ${releaseTag} for ${platformTarget}`);
    const downloadPath = await tc.downloadTool(asset.browser_download_url);

    // Extract the zip file
    core.info('Extracting Sherif binary...');
    const extractedPath = await tc.extractZip(downloadPath);

    // Determine binary name based on platform
    const binaryName = platform === 'win32' ? 'sherif.exe' : 'sherif';
    const binaryPath = path.join(extractedPath, binaryName);

    // Make binary executable on Unix systems
    if (platform !== 'win32') {
      await fsp.chmod(binaryPath, '777');
    }

    // Add to PATH
    core.addPath(extractedPath);

    // Set output
    core.setOutput('sherif-path', binaryPath);
    core.info('Sherif has been installed successfully');

    // Prepare arguments
    if (!additionalArgs) {
      additionalArgs = (await getArgsFromPackageJson()) || '';
    }
    const args = additionalArgs.split(' ').filter(arg => arg !== '');

    // Configure output options to preserve colors
    const options: exec.ExecOptions = {
      ignoreReturnCode: true, // We'll handle the return code ourselves
      env: {
        ...process.env,
        FORCE_COLOR: '3' // Force color output
      }
    };

    // Execute Sherif
    const exitCode = await exec.exec(binaryPath, args, options);

    // Handle exit code
    if (exitCode !== 0) {
      throw new Error(`Sherif execution failed with exit code ${exitCode}`);
    }

  } catch (error) {
    if (error instanceof Error) {
      core.setFailed(error.message);
    } else {
      core.setFailed('An unexpected error occurred');
    }
  }
}

async function getArgsFromPackageJson() {
  try {
    const packageJsonFile = await fsp.readFile(
      path.resolve(process.cwd(), 'package.json')
    );
    const packageJson = JSON.parse(packageJsonFile.toString());

    if (!('scripts' in packageJson)) {
      core.info('No scripts found in package.json');
      return;
    }

    if (!('sherif' in packageJson.scripts)) {
      core.info('No sherif script found in package.json');
      return;
    }

    // Select the args of the sherif script
    const regexResult = /sherif\s([a-zA-Z\s\.-]*)(?=\s&&|$)/g.exec(
      packageJson.scripts.sherif
    );
    if (regexResult && regexResult.length > 1) {
      const args = regexResult[1];
      core.info(`Found args "${args}" package.json`);
      return args;
    }
  } catch {
    core.info('Failed to extract args from package.json');
  }
}

run();
