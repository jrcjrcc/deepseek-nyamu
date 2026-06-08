const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

function getBinaryName(platform) {
  const map = {
    "win32": "deepwhale.exe",
    "darwin": "deepwhale",
    "linux": "deepwhale",
  };
  return map[platform] || "deepwhale";
}

function findBinary() {
  // Check npm-installed location first
  const binaryName = getBinaryName(process.platform);
  const localBin = path.join(__dirname, "..", "bin", binaryName);
  if (fs.existsSync(localBin)) return localBin;
  // Fall back to PATH
  return binaryName;
}

function run(commandName, args) {
  const binary = findBinary();
  if (!fs.existsSync(binary) && !binary.includes("/") && !binary.includes("\\")) {
    // Rely on PATH
    const child = spawn(binary, args, {
      stdio: "inherit",
      env: { ...process.env, NYAMU_MODE: process.env.NYAMU_MODE || "cli" },
    });
    child.on("exit", (code) => process.exit(code ?? 1));
    return;
  }
  if (!fs.existsSync(binary)) {
    console.error(`deepwhale binary not found at ${binary}`);
    console.error("Run `npx deepwhale` or install the native binary manually.");
    process.exit(1);
  }
  const child = spawn(binary, args, {
    stdio: "inherit",
    env: { ...process.env, NYAMU_MODE: process.env.NYAMU_MODE || "cli" },
  });
  child.on("exit", (code) => process.exit(code ?? 1));
}

module.exports = { run };
