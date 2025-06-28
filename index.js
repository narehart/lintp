#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');

function getBinaryName() {
  const platform = os.platform();
  const arch = os.arch();
  
  let binaryName = 'lintp';
  
  if (platform === 'win32') {
    binaryName += '.exe';
  } else if (platform === 'darwin') {
    binaryName += '-macos';
    if (arch === 'arm64') {
      binaryName += '-arm64';
    } else {
      binaryName += '-x64';
    }
  } else if (platform === 'linux') {
    binaryName += '-linux';
    if (arch === 'arm64') {
      binaryName += '-arm64';
    } else {
      binaryName += '-x64';
    }
  }
  
  return binaryName;
}

function main() {
  const binaryName = getBinaryName();
  const binaryPath = path.join(__dirname, 'bin', binaryName);
  
  const child = spawn(binaryPath, process.argv.slice(2), {
    stdio: 'inherit',
    env: process.env
  });
  
  child.on('error', (err) => {
    if (err.code === 'ENOENT') {
      console.error(`Error: Binary not found for your platform (${os.platform()} ${os.arch()})`);
      console.error(`Expected binary at: ${binaryPath}`);
      console.error('Please ensure you have the correct binary for your platform.');
      process.exit(1);
    }
    throw err;
  });
  
  child.on('exit', (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code);
    }
  });
}

if (require.main === module) {
  main();
}

module.exports = { getBinaryName };