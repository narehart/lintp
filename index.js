#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Find the binary
const extension = process.platform === 'win32' ? '.exe' : '';
const binaryName = `lintp${extension}`;
const binaryPath = path.join(__dirname, 'bin', binaryName);

// Check if binary exists
if (!fs.existsSync(binaryPath)) {
  console.error('❌ lintp binary not found.');
  console.error('Please run: npm install lintp');
  process.exit(1);
}

// Forward all arguments to the binary
const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit'
});

child.on('exit', (code) => {
  process.exit(code);
});

child.on('error', (err) => {
  console.error('Failed to start lintp:', err.message);
  process.exit(1);
});