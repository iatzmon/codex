import type { RunOptions, SubagentRunPayload } from "../../ipc/subagents.js";
import { invokeSubagent } from "../../ipc/subagents.js";

export interface RunAgentOptions extends RunOptions {
  format?: "json" | "text";
}

export type RunAgentResult = SubagentRunPayload;

export async function runAgent(
  name: string,
  options: RunAgentOptions = {},
): Promise<RunAgentResult> {
  const payload = await invokeSubagent(name, options);

  if (options.format === "text") {
    renderRunResult(payload);
  }

  return payload;
}

function renderRunResult(result: SubagentRunPayload): void {
  console.log(`Subagent: ${result.name}`);
  if (result.model) {
    console.log(`Model: ${result.model}`);
  }
  if (result.tools.length > 0) {
    console.log(`Tools: ${result.tools.join(", ")}`);
  }
  if (result.summary) {
    console.log(`Summary: ${result.summary}`);
  }
  if (result.detailArtifacts.length > 0) {
    console.log(`Detail: ${result.detailArtifacts[0]}`);
  }
}
