#!/usr/bin/env node

const os = require('os');

console.log('🍵 Installing chaba...');
console.log('');

const platform = os.platform();
const arch = os.arch();

const supported = {
  'darwin-x64': 'macOS Intel',
  'darwin-arm64': 'macOS Apple Silicon',
  'linux-x64': 'Linux x64',
  'linux-arm64': 'Linux ARM64',
  'win32-x64': 'Windows x64'
};

const current = `${platform}-${arch}`;
const platformName = supported[current];

if (!platformName) {
  console.warn('⚠️  Warning: Your platform may not be supported.');
  console.warn(`   Platform: ${platform}`);
  console.warn(`   Architecture: ${arch}`);
  console.warn('');
  console.warn('Supported platforms:');
  Object.entries(supported).forEach(([key, name]) => {
    console.warn(`  - ${name} (${key})`);
  });
  console.warn('');
  console.warn('You can still try to install from source:');
  console.warn('  git clone https://github.com/Nenene01/chaba.git');
  console.warn('  cd chaba && cargo install --path .');
  console.warn('');
} else {
  console.log(`✓ Platform: ${platformName} (${current})`);
  console.log('');
}

console.log('Installation complete!');
console.log('');
console.log('Run "chaba --version" to verify installation.');
console.log('Run "chaba --help" to see available commands.');
console.log('');
console.log('Quick start:');
console.log('  chaba review --pr 123');
console.log('  chaba list');
console.log('');
console.log('Documentation: https://github.com/Nenene01/chaba');
