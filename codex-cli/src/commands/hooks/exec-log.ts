import { fetchHookExecLog } from "../../ipc/hooks.js";

export interface ExecLogOptions {
  since?: string;
  event?: string;
  hookId?: string;
  tail?: number;
  format?: "text" | "json";
}

export interface ExecLogRecord {
  id: string;
  timestamp: string;
  event: string;
  scope: { type: string } & Record<string, unknown>;
  hookId: string;
  decision: {
    decision: string;
    message?: string;
    systemMessage?: string;
    stopReason?: string;
    extra?: unknown;
    exitCode: number;
  };
  durationMs: number;
  stdout: string[];
  stderr: string[];
  error?: string;
  precedenceRank: number;
  payloadHash: string;
  triggerId: string;
}

export async function showExecutionLog(
  options: ExecLogOptions = {},
): Promise<ExecLogRecord[]> {
  const response = await fetchHookExecLog(options);
  let records = response.records;

  if (options.event) {
    records = records.filter((record) => record.event === options.event);
  }

  if (options.hookId) {
    records = records.filter((record) => record.hookId === options.hookId);
  }

  if (typeof options.tail === "number" && options.tail > 0) {
    records = records.slice(-options.tail);
  }

  if (options.format === "text") {
    renderExecLog(records);
  }

  return records;
}

function renderExecLog(records: ExecLogRecord[]): void {
  if (records.length === 0) {
    console.log("No hook executions recorded yet.");
    return;
  }

  const rows = records.map((record) => ({
    Time: record.timestamp,
    Event: record.event,
    Hook: record.hookId,
    Decision: record.decision?.decision ?? "unknown",
  }));

  console.table(rows);
}
