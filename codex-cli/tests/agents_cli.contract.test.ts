import { describe, expect, it } from "vitest";

import { listAgents } from "../src/commands/agents/list.js";
import { runAgent } from "../src/commands/agents/run.js";
import { showAgent } from "../src/commands/agents/show.js";

// Contract expectations derived from specs/004-subagents-md/contracts/agents-list.md

describe("codex agents cli contract", () => {
  it("lists agents in JSON format", async () => {
    const result = await listAgents({ format: "json", includeInvalid: true });
    expect(result).toMatchObject({
      subagents: expect.any(Array),
      invalid: expect.any(Array),
    });
  });

  it("fails to run unknown subagent with a descriptive error", async () => {
    await expect(runAgent("nonexistent-subagent", { format: "json" })).rejects.toThrow(
      /(subagents feature disabled|No subagent named)/i,
    );
  });

  it("fails to show unknown subagent with a descriptive error", async () => {
    await expect(showAgent("nonexistent-subagent", { format: "json" })).rejects.toThrow(
      /No subagent named/i,
    );
  });
});
