import { existsSync, mkdirSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

export type Scope = "project" | "user" | "all";

export interface SubagentRecord {
  name: string;
  scope: Scope;
  description: string;
  tools: string[];
  model: string | null;
  status: string;
  sourcePath: string;
  validationErrors: string[];
}

export interface SubagentListPayload {
  subagents: SubagentRecord[];
  invalid: SubagentRecord[];
}

export interface SubagentRunPayload {
  name: string;
  summary: string | null;
  model: string | null;
  tools: string[];
  detailArtifacts: string[];
}

export interface SubagentErrorPayload {
  error: string;
}

const DEFAULT_COMMAND_TIMEOUT_MS = (() => {
  const value = process.env.CODEX_AGENTS_TIMEOUT_MS;
  if (!value) {
    return 120_000;
  }

  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 120_000;
})();

const KILL_GRACE_PERIOD_MS = 5_000;

type CommandResult = {
  stdout: string;
  stderr: string;
  code: number;
  signal?: NodeJS.Signals | null;
};

type CommandExecutionOptions = {
  timeoutMs?: number;
};

export interface ListOptions {
  scope?: Scope;
  includeInvalid?: boolean;
  timeoutMs?: number;
}

export async function fetchSubagentList(
  options: ListOptions = {},
): Promise<SubagentListPayload> {
  const args = ["agents", "list", "--json"];
  if (options.scope && options.scope !== "all") {
    args.push("--scope", options.scope);
  }
  if (options.includeInvalid) {
    args.push("--invalid");
  }
  const result = await runCodexAgentsCommand(args, {
    timeoutMs: options.timeoutMs,
  });
  ensureSuccess(result, "codex agents list");
  const payload = result.stdout.trim() || "{}";
  return normalizeListPayload(JSON.parse(payload) as RawListPayload);
}

export interface RunOptions {
  tools?: string[];
  timeoutMs?: number;
}

export async function invokeSubagent(
  name: string,
  options: RunOptions = {},
): Promise<SubagentRunPayload> {
  const args = ["agents", "run", name, "--json"];
  for (const tool of options.tools ?? []) {
    args.push("--tool", tool);
  }
  const result = await runCodexAgentsCommand(args, {
    timeoutMs: options.timeoutMs,
  });
  return parseRunResult(result, name);
}

export async function fetchSubagentRecord(name: string): Promise<SubagentRecord> {
  const args = ["agents", "show", name, "--json"];
  const result = await runCodexAgentsCommand(args);
  if (result.code !== 0) {
    throw buildCommandError("codex agents show", result);
  }
  const payload = result.stdout.trim() || "{}";
  return normalizeRecord(JSON.parse(payload) as RawRecordPayload);
}

function ensureSuccess(result: CommandResult, command: string): void {
  if (result.code !== 0) {
    throw buildCommandError(command, result);
  }
}

function buildCommandError(command: string, result: CommandResult): Error {
  const output = result.stderr.trim() || result.stdout.trim();
  const context = output.length > 0 ? `: ${output}` : "";
  const signalSuffix = result.signal ? ` (signal ${result.signal})` : "";
  const error = new Error(
    `${command} failed with exit code ${result.code}${signalSuffix}${context}`,
  );
  (error as { stdout?: string }).stdout = result.stdout;
  (error as { stderr?: string }).stderr = result.stderr;
  (error as { exitCode?: number }).exitCode = result.code;
  if (result.signal) {
    (error as { signal?: NodeJS.Signals | null }).signal = result.signal;
  }
  return error;
}

async function runCodexAgentsCommand(
  args: string[],
  options: CommandExecutionOptions = {},
): Promise<CommandResult> {
  const binary = resolveCodexBinary();
  ensureCodexHome();

  return new Promise<CommandResult>((resolve, reject) => {
    const stdoutChunks: Buffer[] = [];
    const stderrChunks: Buffer[] = [];
    let completed = false;
    let killTimer: NodeJS.Timeout | undefined;

    const resolveTimeoutMs = (): number | undefined => {
      if (typeof options.timeoutMs === "number" && options.timeoutMs > 0) {
        return options.timeoutMs;
      }
      if (options.timeoutMs === 0) {
        return undefined;
      }
      return DEFAULT_COMMAND_TIMEOUT_MS;
    };

    const collectOutput = () => ({
      stdout: Buffer.concat(stdoutChunks).toString("utf8"),
      stderr: Buffer.concat(stderrChunks).toString("utf8"),
    });

    const child = spawn(binary, args, {
      cwd: process.cwd(),
      env: {
        ...process.env,
        CODEX_HOME: process.env.CODEX_HOME ?? defaultCodexHome(),
        CODEX_MANAGED_BY_NPM: "1",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    const onStdout = (chunk: Buffer) => stdoutChunks.push(chunk);
    const onStderr = (chunk: Buffer) => stderrChunks.push(chunk);
    child.stdout.on("data", onStdout);
    child.stderr.on("data", onStderr);

    const clearTimersAndListeners = () => {
      if (timeoutTimer) {
        clearTimeout(timeoutTimer);
        timeoutTimer = undefined;
      }
      if (killTimer) {
        clearTimeout(killTimer);
        killTimer = undefined;
      }
      child.stdout.off("data", onStdout);
      child.stderr.off("data", onStderr);
      child.off("error", onError);
      child.off("close", onClose);
    };

    const settleReject = (error: Error) => {
      if (completed) {
        return;
      }
      completed = true;
      clearTimersAndListeners();
      reject(error);
    };

    const settleResolve = (result: CommandResult) => {
      if (completed) {
        return;
      }
      completed = true;
      clearTimersAndListeners();
      resolve(result);
    };

    const timeoutMs = resolveTimeoutMs();
    let timeoutTimer: NodeJS.Timeout | undefined;
    if (typeof timeoutMs === "number") {
      timeoutTimer = setTimeout(() => {
            const { stdout, stderr } = collectOutput();
            child.kill("SIGTERM");
            killTimer = setTimeout(() => {
              if (!completed) {
                child.kill("SIGKILL");
              }
            }, KILL_GRACE_PERIOD_MS);

            const message: string[] = [
              `codex agents command timed out after ${timeoutMs}ms`,
              `command: ${binary} ${args.join(" ")}`,
            ];
            if (stdout.length > 0) {
              message.push(`stdout:\n${stdout}`);
            }
            if (stderr.length > 0) {
              message.push(`stderr:\n${stderr}`);
            }

            const timeoutError = new Error(message.join("\n\n"));
            (timeoutError as NodeJS.ErrnoException).code = "ETIMEDOUT";
            (timeoutError as { stdout?: string }).stdout = stdout;
            (timeoutError as { stderr?: string }).stderr = stderr;
            (timeoutError as { timeoutMs?: number }).timeoutMs = timeoutMs;

            settleReject(timeoutError);
          }, timeoutMs);
    }

    const onError = (err: Error) => {
      const { stdout, stderr } = collectOutput();
      const error = new Error(
        [`Failed to launch codex agents command: ${binary} ${args.join(" ")}`, err.message]
          .filter(Boolean)
          .join("\n"),
      );
      (error as { cause?: Error }).cause = err;
      (error as { stdout?: string }).stdout = stdout;
      (error as { stderr?: string }).stderr = stderr;
      settleReject(error);
    };

    const onClose = (code: number | null, signal: NodeJS.Signals | null) => {
      const { stdout, stderr } = collectOutput();
      settleResolve({
        stdout,
        stderr,
        code: code ?? 1,
        signal,
      });
    };

    child.on("error", onError);
    child.on("close", onClose);
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

function ensureCodexHome(): void {
  const home = process.env.CODEX_HOME ?? defaultCodexHome();
  if (!existsSync(home)) {
    mkdirSync(home, { recursive: true });
  }
}

function defaultCodexHome(): string {
  return path.join(os.homedir(), ".codex");
}

function binaryName(): string {
  const platform = os.platform();
  if (platform === "win32") {
    return "codex.exe";
  }
  return "codex";
}

function fileDirectory(): string {
  return path.dirname(fileURLToPath(import.meta.url));
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

type RawRecordPayload = {
  name?: string;
  scope?: string;
  description?: string;
  tools?: string[];
  model?: string | null;
  status?: string;
  source_path?: string;
  sourcePath?: string;
  validation_errors?: string[];
  validationErrors?: string[];
};

type RawListPayload = {
  subagents?: RawRecordPayload[];
  invalid?: RawRecordPayload[];
};

type RawRunPayload = {
  name?: string;
  summary?: string | null;
  model?: string | null;
  tools?: string[];
  detail_artifacts?: string[];
  detailArtifacts?: string[];
  error?: string;
};

function normalizeListPayload(raw: RawListPayload): SubagentListPayload {
  const normalizeEntries = (records: RawRecordPayload[] | undefined): SubagentRecord[] =>
    (records ?? []).map((record) => normalizeRecord(record));

  return {
    subagents: normalizeEntries(raw.subagents),
    invalid: normalizeEntries(raw.invalid),
  };
}

function normalizeRecord(raw: RawRecordPayload): SubagentRecord {
  return {
    name: raw.name ?? "",
    scope: normalizeScope(raw.scope),
    description: raw.description ?? "",
    tools: raw.tools ?? [],
    model: raw.model ?? null,
    status: raw.status ?? "unknown",
    sourcePath: raw.sourcePath ?? raw.source_path ?? "",
    validationErrors: raw.validationErrors ?? raw.validation_errors ?? [],
  };
}

function normalizeScope(value?: string): Scope {
  switch (value) {
    case "project":
      return "project";
    case "user":
      return "user";
    default:
      return "all";
  }
}

function parseRunResult(result: CommandResult, name: string): SubagentRunPayload {
  const payloadText = result.stdout.trim();
  if (result.code === 0) {
    if (!payloadText) {
      return {
        name,
        summary: null,
        model: null,
        tools: [],
        detailArtifacts: [],
      };
    }
    const raw = JSON.parse(payloadText) as RawRunPayload;
    return normalizeRunPayload(raw);
  }

  if (payloadText.length > 0) {
    const raw = JSON.parse(payloadText) as RawRunPayload;
    const message = raw.error ?? payloadText;
    throw new Error(message);
  }

  throw buildCommandError("codex agents run", result);
}

function normalizeRunPayload(raw: RawRunPayload): SubagentRunPayload {
  return {
    name: raw.name ?? "",
    summary: raw.summary ?? null,
    model: raw.model ?? null,
    tools: raw.tools ?? [],
    detailArtifacts: raw.detailArtifacts ?? raw.detail_artifacts ?? [],
  };
}
