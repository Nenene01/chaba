#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');

/**
 * Get the platform-specific binary path
 */
function getPlatformBinary() {
  const platform = os.platform();
  const arch = os.arch();

  // Map platform and architecture to package names
  const platformMap = {
    'darwin-x64': '@nenene01/chaba-darwin-x64',
    'darwin-arm64': '@nenene01/chaba-darwin-arm64',
    'linux-x64': '@nenene01/chaba-linux-x64',
    'linux-arm64': '@nenene01/chaba-linux-arm64',
    'win32-x64': '@nenene01/chaba-win32-x64'
  };

  const key = `${platform}-${arch}`;
  const packageName = platformMap[key];

  if (!packageName) {
    console.error(`❌ Unsupported platform: ${platform}-${arch}`);
    console.error('');
    console.error('Supported platforms:');
    Object.keys(platformMap).forEach(p => console.error(`  - ${p}`));
    console.error('');
    console.error('For other platforms, please install from source:');
    console.error('  git clone https://github.com/Nenene01/chaba.git');
    console.error('  cd chaba && cargo install --path .');
    process.exit(1);
  }

  try {
    // Try to resolve the binary from the platform package
    const binaryName = platform === 'win32' ? 'chaba.exe' : 'chaba';
    const binaryPath = require.resolve(`${packageName}/${binaryName}`);
    return binaryPath;
  } catch (error) {
    console.error(`❌ Binary not found for ${platform}-${arch}`);
    console.error('');
    console.error('This usually means the installation failed.');
    console.error('Please try reinstalling:');
    console.error('  npm install -g @nenene01/chaba');
    console.error('');
    console.error('Or install from source:');
    console.error('  cargo install --git https://github.com/Nenene01/chaba');
    process.exit(1);
  }
}

/**
 * Main execution
 */
function main() {
  const binary = getPlatformBinary();

  // Spawn the binary with the same arguments
  const child = spawn(binary, process.argv.slice(2), {
    stdio: 'inherit',
    windowsHide: true
  });

  // Handle process exit
  child.on('error', (error) => {
    console.error(`❌ Failed to execute chaba: ${error.message}`);
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code || 0);
    }
  });

  // Forward signals
  process.on('SIGINT', () => {
    child.kill('SIGINT');
  });

  process.on('SIGTERM', () => {
    child.kill('SIGTERM');
  });
}

main();
