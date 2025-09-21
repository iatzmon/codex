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

type RegistryEvent = ListHooksResult["events"][number];

const PLACEHOLDER_LAYERS: HookLayerSummary[] = [
  {
    scope: "managed",
    path: "/etc/codex/hooks/policy.toml",
    checksum: "0000000000000000",
    loadedHooks: 0,
    skippedHooks: [],
  },
];

const PLACEHOLDER_EVENTS: RegistryEvent[] = [
  {
    event: "PreToolUse",
    hooks: [],
  },
];

const PLACEHOLDER_RECORDS: ExecLogRecord[] = [
  {
    event: "PreToolUse",
    hookId: "managed.placeholder.guard",
    decision: "allow",
    timestamp: new Date(0).toISOString(),
  },
];

export async function fetchHookRegistrySnapshot(
  _options?: ListHooksOptions,
): Promise<HookRegistrySnapshot> {
  return {
    layers: PLACEHOLDER_LAYERS,
    events: PLACEHOLDER_EVENTS,
  };
}

export async function fetchHookExecLog(
  _options?: ExecLogOptions,
): Promise<HookExecLogResponse> {
  return { records: PLACEHOLDER_RECORDS };
}

export async function requestHookValidation(
  _options?: ValidateOptions,
): Promise<HookValidationResponse> {
  return {
    status: "ok",
    layers: PLACEHOLDER_LAYERS,
  };
}

export async function requestHookReload(): Promise<HookReloadResponse> {
  return { reloaded: false, message: "Hook reload not yet implemented" };
}
