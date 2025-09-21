export interface ExecLogOptions {
  since?: string;
  event?: string;
  hookId?: string;
  tail?: number;
  format?: "text" | "json";
}

export interface ExecLogRecord {
  event: string;
  hookId: string;
  decision: string;
  timestamp: string;
}

export async function showExecutionLog(
  _options: ExecLogOptions,
): Promise<ExecLogRecord[]> {
  throw new Error("showExecutionLog not implemented");
}
