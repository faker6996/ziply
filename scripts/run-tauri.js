#!/usr/bin/env node

const { existsSync } = require("node:fs");
const { spawn, spawnSync } = require("node:child_process");
const { delimiter, join } = require("node:path");
const { homedir } = require("node:os");

const tauriArgs = process.argv.slice(2);

if (tauriArgs.length === 0) {
  console.error("Usage: node scripts/run-tauri.js <tauri-subcommand> [...args]");
  process.exit(1);
}

const repoRoot = join(__dirname, "..");
const tauriConfigPath = join(repoRoot, "src-tauri", "tauri.conf.json");
const cargoBinDirectory = join(homedir(), ".cargo", "bin");
const envWithCargo = {
  ...process.env,
  PATH: prependPathEntry(process.env.PATH, cargoBinDirectory),
};

if (process.platform !== "win32") {
  runNpmExecTauri(envWithCargo);
} else if (isWindowsBuildEnvironmentReady(envWithCargo)) {
  runNpmExecTauri(envWithCargo);
} else {
  const vsDevCmd = resolveVsDevCmd();

  if (!vsDevCmd) {
    console.error(
      [
        "Unable to find a Visual Studio developer shell for Tauri.",
        "Set VSDEVCMD_PATH or install Visual Studio Build Tools 2022.",
      ].join(" "),
    );
    process.exit(1);
  }

  const command = [
      `call "${vsDevCmd}" -arch=x64 -host_arch=x64 >nul`,
      `set "PATH=${cargoBinDirectory};%PATH%"`,
      windowsNpmExecCommand(
        `exec --workspace @ziply/desktop tauri -- ${toCmdArgsString(tauriArgs)} --config ${quoteForCmd(tauriConfigPath)}`,
      ),
  ].join(" && ");

  const child = spawn("cmd.exe", ["/d", "/c", command], {
    cwd: repoRoot,
    env: process.env,
    stdio: "inherit",
    windowsVerbatimArguments: true,
  });

  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }

    process.exit(code ?? 1);
  });
}

function runNpmExecTauri(env) {
  const invocation = npmExecInvocation();
  const child = spawn(
    invocation.command,
    [
      ...invocation.args,
      "exec",
      "--workspace",
      "@ziply/desktop",
      "tauri",
      "--",
      ...tauriArgs,
      "--config",
      tauriConfigPath,
    ],
    {
      cwd: repoRoot,
      env,
      stdio: "inherit",
      shell: false,
    },
  );

  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }

    process.exit(code ?? 1);
  });
}

function npmCommand() {
  return process.platform === "win32" ? "npm.cmd" : "npm";
}

function npmExecInvocation() {
  const npmExecPath = process.env.npm_execpath;
  if (npmExecPath && /\.c?js$/i.test(npmExecPath)) {
    return {
      command: process.execPath,
      args: [npmExecPath],
    };
  }

  return {
    command: npmCommand(),
    args: [],
  };
}

function prependPathEntry(currentPath, entry) {
  if (!entry || !existsSync(entry)) {
    return currentPath || "";
  }

  const pathEntries = (currentPath || "").split(delimiter).filter(Boolean);
  if (pathEntries.some((candidate) => candidate.toLowerCase() === entry.toLowerCase())) {
    return currentPath || "";
  }

  return [entry, ...pathEntries].join(delimiter);
}

function isWindowsBuildEnvironmentReady(env) {
  if (env.VSCMD_VER) {
    return true;
  }

  return commandExists("cargo.exe", env) && commandExists("link.exe", env);
}

function commandExists(command, env) {
  const result = spawnSync("where.exe", [command], {
    env,
    stdio: "ignore",
    windowsHide: true,
  });

  return result.status === 0;
}

function resolveVsDevCmd() {
  const candidates = [
    process.env.VSDEVCMD_PATH,
    "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\Common7\\Tools\\VsDevCmd.bat",
    "C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\Common7\\Tools\\VsDevCmd.bat",
    "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\Community\\Common7\\Tools\\VsDevCmd.bat",
    "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\Common7\\Tools\\VsDevCmd.bat",
    "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\Professional\\Common7\\Tools\\VsDevCmd.bat",
    "C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional\\Common7\\Tools\\VsDevCmd.bat",
    "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\Enterprise\\Common7\\Tools\\VsDevCmd.bat",
    "C:\\Program Files\\Microsoft Visual Studio\\2022\\Enterprise\\Common7\\Tools\\VsDevCmd.bat",
  ];

  return candidates.find((candidate) => candidate && existsSync(candidate));
}

function toCmdArgsString(args) {
  return args.map(quoteForCmd).join(" ");
}

function quoteForCmd(value) {
  if (/^[A-Za-z0-9_./:=+-]+$/.test(value)) {
    return value;
  }

  return `"${value.replace(/"/g, '""')}"`;
}

function windowsNpmExecCommand(trailingArgs) {
  const npmExecPath = process.env.npm_execpath;
  if (npmExecPath && /\.c?js$/i.test(npmExecPath)) {
    return `${quoteForCmd(process.execPath)} ${quoteForCmd(npmExecPath)} ${trailingArgs}`;
  }

  return `npm ${trailingArgs}`;
}
