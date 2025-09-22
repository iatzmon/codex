import type { SubagentRecord } from "../../ipc/subagents.js";
import { fetchSubagentRecord } from "../../ipc/subagents.js";

export interface ShowAgentOptions {
  format?: "json" | "text";
}

export type ShowAgentResult = SubagentRecord;

export async function showAgent(
  name: string,
  options: ShowAgentOptions = {},
): Promise<ShowAgentResult> {
  const record = await fetchSubagentRecord(name);

  if (options.format === "text") {
    renderAgent(record);
  }

  return record;
}

function renderAgent(record: SubagentRecord): void {
  console.log(`Name: ${record.name}`);
  console.log(`Scope: ${record.scope}`);
  console.log(`Description: ${record.description}`);
  console.log(`Tools: ${record.tools.join(", ") || "(all)"}`);
  console.log(`Model: ${record.model ?? "(inherit)"}`);
  console.log(`Status: ${record.status}`);
  console.log(`Source: ${record.sourcePath}`);
  if (record.validationErrors.length > 0) {
    console.log(`Validation errors: ${record.validationErrors.join("; ")}`);
  }
}
