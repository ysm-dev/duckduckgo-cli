const childProcess = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

function binaryPath() {
  const ext = process.platform === "win32" ? ".exe" : "";
  return require.resolve(`@duckduckgo-cli/${platformName()}/bin/duckduckgo${ext}`);
}

function platformName() {
  const osName = process.platform;
  const arch = process.arch === "x64" ? "x64" : process.arch === "arm64" ? "arm64" : null;
  if (!arch) return `${osName}-${process.arch}`;
  if (osName === "darwin" || osName === "win32") return `${osName}-${arch}`;
  if (osName === "linux") return `linux-${arch}-${linuxLibc()}`;
  return `${osName}-${arch}`;
}

function linuxLibc() {
  const report = process.report && process.report.getReport && process.report.getReport();
  if (report && report.header && report.header.glibcVersionRuntime) return "gnu";
  const cache = path.join(process.env.XDG_CACHE_HOME || path.join(os.homedir(), ".cache"), "duckduckgo-cli", "libc");
  try {
    return fs.readFileSync(cache, "utf8").trim() || "musl";
  } catch (_) {
    const probed = probeLibc();
    fs.mkdirSync(path.dirname(cache), { recursive: true });
    fs.writeFileSync(cache, probed);
    return probed;
  }
}

function probeLibc() {
  try {
    const result = childProcess.spawnSync("ldd", ["--version"], { encoding: "utf8" });
    return `${result.stdout}${result.stderr}`.toLowerCase().includes("musl") ? "musl" : "gnu";
  } catch (_) {
    return "musl";
  }
}

module.exports = { binaryPath, platformName };
