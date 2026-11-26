#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(scriptDir, "..");
const tauriDir = path.join(rootDir, "src-tauri");

const errors = [];

function fail(message) {
  errors.push(message);
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function readCargoVersion(cargoTomlPath) {
  const content = fs.readFileSync(cargoTomlPath, "utf8").replace(/\r\n/g, "\n");
  const pkgSection = content.split(/^\[/m).find((section) => section.startsWith("package]"));
  if (!pkgSection) {
    fail("Could not find [package] section in src-tauri/Cargo.toml");
    return null;
  }
  const match = pkgSection.match(/^version\s*=\s*"([^"]+)"/m);
  if (!match) {
    fail("Could not find version in [package] section of src-tauri/Cargo.toml");
    return null;
  }
  return match[1];
}

const semverRegex = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

const pkgJson = readJson(path.join(rootDir, "package.json"));
const tauriConf = readJson(path.join(tauriDir, "tauri.conf.json"));
const cargoVersion = readCargoVersion(path.join(tauriDir, "Cargo.toml"));

const versions = {
  "package.json": pkgJson.version,
  "src-tauri/tauri.conf.json": tauriConf.version,
  "src-tauri/Cargo.toml": cargoVersion,
};

const packageLockPath = path.join(rootDir, "package-lock.json");
if (fs.existsSync(packageLockPath)) {
  const pkgLock = readJson(packageLockPath);
  versions["package-lock.json"] = pkgLock.version;
  if (pkgLock.packages?.[""]) {
    versions['package-lock.json (packages[""])'] = pkgLock.packages[""].version;
  }
}

for (const [label, version] of Object.entries(versions)) {
  if (!version || !semverRegex.test(version)) {
    fail(`Invalid SemVer in ${label}: ${String(version)}`);
  }
}

const mismatched = Object.entries(versions).filter(
  ([, version]) => version !== pkgJson.version,
);
if (mismatched.length > 0) {
  const summary = Object.entries(versions)
    .map(([label, version]) => `${label}=${String(version)}`)
    .join(", ");
  fail(`Version mismatch detected: ${summary}`);
}

if (!fs.existsSync(path.join(rootDir, "LICENSE"))) {
  fail("Missing LICENSE file in repository root");
}

if (!tauriConf.identifier || tauriConf.identifier.includes("tauri.dev")) {
  fail(`Invalid Tauri bundle identifier: ${String(tauriConf.identifier)}`);
}

if (!tauriConf.app?.security?.csp) {
  fail("app.security.csp must be set in src-tauri/tauri.conf.json for release builds");
}

const bundle = tauriConf.bundle ?? {};
if (!bundle.active) {
  fail("bundle.active must be true in src-tauri/tauri.conf.json");
}

if (!bundle.publisher || !bundle.shortDescription || !bundle.license) {
  fail("bundle.publisher, bundle.shortDescription, and bundle.license must be configured");
}

const requiredTauriAssets = [
  ...(bundle.icon ?? []),
  bundle.licenseFile,
  bundle.linux?.deb?.desktopTemplate,
  bundle.linux?.rpm?.desktopTemplate,
  bundle.windows?.nsis?.installerIcon,
  bundle.windows?.nsis?.uninstallerIcon,
].filter(Boolean);

for (const relPath of requiredTauriAssets) {
  const resolved = path.resolve(tauriDir, relPath);
  if (!fs.existsSync(resolved)) {
    fail(`Missing bundle asset referenced in tauri.conf.json: ${relPath} (resolved: ${resolved})`);
  }
}

if (errors.length > 0) {
  console.error("Release readiness check failed:\n");
  for (const err of errors) {
    console.error(`  - ${err}`);
  }
  process.exit(1);
}

console.log(`Release readiness check passed (v${pkgJson.version}).`);
