#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const newVersion = process.argv[2]?.replace(/^v/, "");
const semverRegex = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

if (!newVersion || !semverRegex.test(newVersion)) {
  console.error("Usage: npm run release:version -- <semver>  (e.g. 0.2.0)");
  process.exit(1);
}

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(scriptDir, "..");
const tauriDir = path.join(rootDir, "src-tauri");

const cargoTomlPath = path.join(tauriDir, "Cargo.toml");
const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
const cargoVersionPattern = /(\[package\](?:(?!^\[)[\s\S])*?^version\s*=\s*")([^"]+)(")/m;
const cargoMatch = cargoToml.match(cargoVersionPattern);
if (!cargoMatch) {
  console.error("Failed to locate [package] version in src-tauri/Cargo.toml");
  process.exit(1);
}

function updateJsonVersion(filePath, updater) {
  const original = fs.readFileSync(filePath, "utf8");
  const data = JSON.parse(original);
  updater(data);
  const updated = `${JSON.stringify(data, null, 2)}\n`;
  if (updated !== original) {
    fs.writeFileSync(filePath, updated, "utf8");
  }
}

const touchedFiles = ["package.json"];
updateJsonVersion(path.join(rootDir, "package.json"), (pkg) => {
  pkg.version = newVersion;
});

const packageLockPath = path.join(rootDir, "package-lock.json");
if (fs.existsSync(packageLockPath)) {
  touchedFiles.push("package-lock.json");
  updateJsonVersion(packageLockPath, (lock) => {
    lock.version = newVersion;
    if (lock.packages?.[""]) {
      lock.packages[""].version = newVersion;
    }
  });
}

touchedFiles.push("src-tauri/tauri.conf.json");
updateJsonVersion(path.join(tauriDir, "tauri.conf.json"), (conf) => {
  conf.version = newVersion;
});

touchedFiles.push("src-tauri/Cargo.toml");
if (cargoMatch[2] !== newVersion) {
  const updatedCargoToml = cargoToml.replace(
    cargoVersionPattern,
    `$1${newVersion}$3`,
  );
  fs.writeFileSync(cargoTomlPath, updatedCargoToml, "utf8");
}

console.log(`Synced version to ${newVersion} across ${touchedFiles.join(", ")}.`);
console.log("");
console.log("Next steps:");
console.log(`  1. Update CHANGELOG.md for v${newVersion}`);
console.log("  2. Refresh src-tauri/Cargo.lock with: cargo check --manifest-path src-tauri/Cargo.toml");
console.log("  3. Verify with: npm run release:check");
console.log(`  4. Commit & tag: git tag -a v${newVersion} -m "Release v${newVersion}" && git push origin main --follow-tags`);
