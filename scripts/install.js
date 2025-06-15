#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

const GITHUB_REPO = 'narehart/lintp';
const VERSION = require('../package.json').version;

function getPlatformInfo() {
  const platform = os.platform();
  const arch = os.arch();
  
  let platformName;
  let archName;
  let extension = '';
  
  switch (platform) {
    case 'darwin':
      platformName = 'apple-darwin';
      break;
    case 'linux':
      platformName = 'unknown-linux-gnu';
      break;
    case 'win32':
      platformName = 'pc-windows-msvc';
      extension = '.exe';
      break;
    default:
      throw new Error(`Unsupported platform: ${platform}`);
  }
  
  switch (arch) {
    case 'x64':
      archName = 'x86_64';
      break;
    case 'arm64':
      archName = 'aarch64';
      break;
    default:
      throw new Error(`Unsupported architecture: ${arch}`);
  }
  
  return {
    target: `${archName}-${platformName}`,
    extension,
    platform,
    arch
  };
}

function downloadBinary() {
  const { target, extension } = getPlatformInfo();
  const binaryName = `lintp${extension}`;
  const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${binaryName}-${target}`;
  
  const binDir = path.join(__dirname, '..', 'bin');
  const binaryPath = path.join(binDir, 'lintp' + extension);
  
  // Create bin directory if it doesn't exist
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }
  
  console.log(`Downloading lintp binary for ${target}...`);
  console.log(`URL: ${downloadUrl}`);
  
  try {
    // Try to download the pre-built binary
    execSync(`curl -L -o "${binaryPath}" "${downloadUrl}"`, { stdio: 'inherit' });
    
    // Make the binary executable (Unix-like systems)
    if (process.platform !== 'win32') {
      execSync(`chmod +x "${binaryPath}"`);
    }
    
    console.log('✅ lintp binary installed successfully!');
    console.log(`Binary location: ${binaryPath}`);
  } catch (error) {
    console.error('❌ Failed to download pre-built binary.');
    console.log('This might be because:');
    console.log('1. No pre-built binary exists for your platform');
    console.log('2. The release has not been published yet');
    console.log('3. Network connectivity issues');
    console.log('');
    console.log('You can build from source instead:');
    console.log('1. Install Rust: https://rustup.rs/');
    console.log('2. Clone the repository: git clone https://github.com/narehart/lintp.git');
    console.log('3. Build: cargo build --release');
    console.log('4. Copy target/release/lintp to your PATH');
    
    process.exit(1);
  }
}

function main() {
  try {
    console.log('Installing lintp...');
    downloadBinary();
  } catch (error) {
    console.error('Installation failed:', error.message);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}