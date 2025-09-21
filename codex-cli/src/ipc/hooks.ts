import type { ExecLogRecord } from "../commands/hooks/exec-log.js";
import type { ListHooksResult } from "../commands/hooks/list.js";
import type { ValidateSummary } from "../commands/hooks/validate.js";

export interface HookRegistrySnapshot extends ListHooksResult {}

export interface HookValidationResponse extends ValidateSummary {}

export interface HookExecLogResponse {
  records: ExecLogRecord[];
}

export async function fetchHookRegistrySnapshot(): Promise<HookRegistrySnapshot> {
  throw new Error("fetchHookRegistrySnapshot not implemented");
}

export async function fetchHookExecLog(): Promise<HookExecLogResponse> {
  throw new Error("fetchHookExecLog not implemented");
}

export async function requestHookValidation(): Promise<HookValidationResponse> {
  throw new Error("requestHookValidation not implemented");
}
