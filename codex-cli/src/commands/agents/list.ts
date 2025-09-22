import type { ListOptions, SubagentListPayload, SubagentRecord } from "../../ipc/subagents.js";
import { fetchSubagentList } from "../../ipc/subagents.js";

export interface ListAgentsOptions extends ListOptions {
  format?: "json" | "text";
}

export interface ListAgentsResult extends SubagentListPayload {}

export async function listAgents(options: ListAgentsOptions = {}): Promise<ListAgentsResult> {
  const payload = await fetchSubagentList({
    scope: options.scope,
    includeInvalid: options.includeInvalid,
  });

  if (options.format === "text") {
    renderAgentsList(payload, options.includeInvalid ?? false);
  }

  return payload;
}

function renderAgentsList(result: SubagentListPayload, includeInvalid: boolean): void {
  if (result.subagents.length === 0 && (!includeInvalid || result.invalid.length === 0)) {
    console.log("No subagents found.");
    return;
  }

  if (result.subagents.length > 0) {
    console.log("Available subagents:\n");
    const rows = result.subagents.map((record) => formatRecord(record));
    console.table(rows);
  }

  if (includeInvalid && result.invalid.length > 0) {
    console.log("\nInvalid subagents:\n");
    const rows = result.invalid.map((record) => ({
      ...formatRecord(record),
      Errors: record.validationErrors.join("; ") || "(none)",
    }));
    console.table(rows);
  }
}

function formatRecord(record: SubagentRecord): Record<string, unknown> {
  return {
    Name: record.name,
    Scope: record.scope,
    Description: record.description,
    Tools: record.tools.join(", ") || "(all)",
    Model: record.model ?? "(inherit)",
    Status: record.status,
  };
}
