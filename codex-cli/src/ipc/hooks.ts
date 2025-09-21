import { existsSync, mkdirSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

import type { ExecLogOptions, ExecLogRecord } from "../commands/hooks/exec-log.js";
import type { ListHooksOptions, ListHooksResult } from "../commands/hooks/list.js";
import type { ValidateOptions, ValidateSummary } from "../commands/hooks/validate.js";

export interface HookLayerSummary {
  scope: "managed" | "project" | "local";
  path: string;
  checksum: string;
  loadedHooks: number;
  skippedHooks: Array<{ hookId?: string; reason: string; details?: string }>;
}

export interface HookRegistrySnapshot extends ListHooksResult {}

export interface HookValidationResponse extends ValidateSummary {}

export interface HookExecLogResponse {
  records: ExecLogRecord[];
}

export interface HookReloadResponse {
  reloaded: boolean;
  message?: string;
}

type CommandResult = {
  stdout: string;
  stderr: string;
  code: number;
};

type RawHookScope = { type: string } & Record<string, unknown>;

type RawHookDefinition = {
  id: string;
  scope?: RawHookScope;
  command?: string[];
  notes?: string;
};

type RawSkippedHook = {
  hookId?: string;
  hook_id?: string;
  reason?: string;
  details?: string;
};

type RawLayerSummary = {
  scope?: RawHookScope;
  path?: string;
  checksum?: string;
  loadedHooks?: number;
  loaded_hooks?: number;
  skippedHooks?: RawSkippedHook[];
  skipped_hooks?: RawSkippedHook[];
};

type RawRegistrySnapshot = {
  events?: Record<string, RawHookDefinition[]>;
  layers?: RawLayerSummary[];
};

export async function fetchHookRegistrySnapshot(
  options: ListHooksOptions = {},
): Promise<HookRegistrySnapshot> {
  const args = ["hooks", "list", "--json"];
  if (options.event) {
    args.push("--event", options.event);
  }
  if (options.scope) {
    args.push("--scope", options.scope);
  }
  const result = await runCodexHooksCommand(args);
  ensureSuccess(result, "codex hooks list");
  const trimmed = result.stdout.trim() || "{}";
  const raw = JSON.parse(trimmed) as RawRegistrySnapshot;
  return normalizeRegistrySnapshot(raw);
}

export async function fetchHookExecLog(
  options: ExecLogOptions = {},
): Promise<HookExecLogResponse> {
  const args = ["hooks", "exec-log", "--json"];
  if (options.since) {
    args.push("--since", options.since);
  }
  if (options.event) {
    args.push("--event", options.event);
  }
  if (options.hookId) {
    args.push("--hook-id", options.hookId);
  }
  if (typeof options.tail === "number") {
    args.push("--tail", String(options.tail));
  }
  const result = await runCodexHooksCommand(args);
  ensureSuccess(result, "codex hooks exec-log");
  const payload = result.stdout.trim();
  const records = payload.length > 0 ? (JSON.parse(payload) as ExecLogRecord[]) : [];
  return { records };
}

export async function requestHookValidation(
  options: ValidateOptions = {},
): Promise<HookValidationResponse> {
  const args = ["hooks", "validate", "--json"];
  if (options.scope) {
    args.push("--scope", options.scope);
  }
  const result = await runCodexHooksCommand(args);
  // Exit codes 0/3/2 indicate ok/warning/error respectively. The CLI embeds the
  // status in JSON, so always parse the payload even on non-zero exit.
  const trimmed = result.stdout.trim() || "{}";
  const summary = JSON.parse(trimmed) as HookValidationResponse;
  return summary;
}

export async function requestHookReload(): Promise<HookReloadResponse> {
  const result = await runCodexHooksCommand(["hooks", "reload"]);
  const message = (result.stdout || result.stderr).trim();
  return {
    reloaded: result.code === 0 && message.toLowerCase().includes("reloaded"),
    message: message.length > 0 ? message : undefined,
  };
}

function ensureSuccess(result: CommandResult, command: string): void {
  if (result.code !== 0) {
    const errorOutput = result.stderr.trim() || result.stdout.trim();
    const context = errorOutput.length > 0 ? `: ${errorOutput}` : "";
    throw new Error(`${command} failed with exit code ${result.code}${context}`);
  }
}

async function runCodexHooksCommand(args: string[]): Promise<CommandResult> {
  const binary = resolveCodexBinary();
  ensureCodexHome();

  return new Promise<CommandResult>((resolve, reject) => {
    const stdoutChunks: Buffer[] = [];
    const stderrChunks: Buffer[] = [];

    const child = spawn(binary, args, {
      cwd: process.cwd(),
      env: {
        ...process.env,
        CODEX_HOME: process.env.CODEX_HOME ?? defaultCodexHome(),
        CODEX_NO_JS_HOOKS_SHIM: "1",
        CODEX_MANAGED_BY_NPM: "1",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    child.stdout.on("data", (chunk) => stdoutChunks.push(chunk));
    child.stderr.on("data", (chunk) => stderrChunks.push(chunk));

    child.on("error", (err) => {
      reject(err);
    });

    child.on("close", (code) => {
      resolve({
        stdout: Buffer.concat(stdoutChunks).toString("utf8"),
        stderr: Buffer.concat(stderrChunks).toString("utf8"),
        code: code ?? 1,
      });
    });
  });
}

function resolveCodexBinary(): string {
  const envBin =
    process.env.CODEX_CLI_BIN ??
    process.env.CODEX_CLI_BINARY ??
    process.env.CODEX_DEV_CLI_BIN;
  if (envBin && existsSync(envBin)) {
    return envBin;
  }

  const cliRoot = path.resolve(fileDirectory(), "..", "..");
  const workspaceRoot = path.resolve(cliRoot, "..");

  const workspaceDebug = path.resolve(
    workspaceRoot,
    "codex-rs",
    "target",
    "debug",
    binaryName(),
  );
  if (existsSync(workspaceDebug)) {
    return workspaceDebug;
  }

  const workspaceRelease = path.resolve(
    workspaceRoot,
    "codex-rs",
    "target",
    "release",
    binaryName(),
  );
  if (existsSync(workspaceRelease)) {
    return workspaceRelease;
  }

  const triple = detectTargetTriple();
  if (triple) {
    const packaged = path.resolve(cliRoot, "bin", `codex-${triple}`);
    if (existsSync(packaged)) {
      return packaged;
    }
  }

  throw new Error(
    "Unable to locate Codex CLI binary. Set CODEX_CLI_BIN to the compiled codex executable.",
  );
}

function detectTargetTriple(): string | null {
  const platform = os.platform();
  const arch = os.arch();

  if (platform === "linux" || platform === "android") {
    if (arch === "x64") {
      return "x86_64-unknown-linux-musl";
    }
    if (arch === "arm64") {
      return "aarch64-unknown-linux-musl";
    }
    return null;
  }

  if (platform === "darwin") {
    if (arch === "x64") {
      return "x86_64-apple-darwin";
    }
    if (arch === "arm64") {
      return "aarch64-apple-darwin";
    }
    return null;
  }

  if (platform === "win32") {
    if (arch === "x64") {
      return "x86_64-pc-windows-msvc.exe";
    }
    if (arch === "arm64") {
      return "aarch64-pc-windows-msvc.exe";
    }
    return null;
  }

  return null;
}

function binaryName(): string {
  return process.platform === "win32" ? "codex.exe" : "codex";
}

function fileDirectory(): string {
  return path.dirname(fileURLToPath(import.meta.url));
}

function defaultCodexHome(): string {
  const cliRoot = path.resolve(fileDirectory(), "..", "..");
  const workspaceRoot = path.resolve(cliRoot, "..");
  return path.join(workspaceRoot, ".codex");
}

function ensureCodexHome(): void {
  const home = defaultCodexHome();
  if (!existsSync(home)) {
    mkdirSync(home, { recursive: true });
  }
  const logsDir = path.join(home, "logs");
  if (!existsSync(logsDir)) {
    mkdirSync(logsDir, { recursive: true });
  }
}

function normalizeRegistrySnapshot(raw: RawRegistrySnapshot): HookRegistrySnapshot {
  const layers = (raw.layers ?? []).map((layer) => ({
    scope: normalizeScope(layer.scope),
    path: typeof layer.path === "string" ? layer.path : "",
    checksum: layer.checksum ?? "",
    loadedHooks: layer.loadedHooks ?? layer.loaded_hooks ?? 0,
    skippedHooks: (layer.skippedHooks ?? layer.skipped_hooks ?? []).map((skipped) => ({
      hookId: skipped.hookId ?? skipped.hook_id,
      reason: skipped.reason ?? "",
      details: skipped.details,
    })),
  }));

  const eventsEntries = Object.entries(raw.events ?? {});
  const events = eventsEntries.map(([event, hooks]) => ({
    event,
    hooks: hooks.map((hook) => ({
      id: hook.id,
      scope: normalizeScope(hook.scope),
      command: hook.command ?? [],
    })),
  }));

  return { layers, events };
}

function normalizeScope(scope: RawHookScope | undefined): "managed" | "project" | "local" {
  if (!scope || typeof scope.type !== "string") {
    return "managed";
  }
  switch (scope.type) {
    case "project":
      return "project";
    case "localUser":
      return "local";
    case "managedPolicy":
    default:
      return "managed";
  }
}
