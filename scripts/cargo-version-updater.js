const fs = require("fs");

module.exports.readVersion = function (contents) {
  const match = contents.match(/version\s*=\s*"([^"]+)"/);
  if (match) {
    return match[1];
  }
  throw new Error("Could not find version in Cargo.toml");
};

module.exports.writeVersion = function (contents, version) {
  return contents.replace(/version\s*=\s*"[^"]+"/, `version = "${version}"`);
};
