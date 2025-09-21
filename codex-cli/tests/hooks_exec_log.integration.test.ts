import { describe, expect, it } from "vitest";

import { showExecutionLog } from "../src/commands/hooks/exec-log.js";

describe("codex hooks exec-log integration", () => {
  it("tails execution log for most recent events", async () => {
    const records = await showExecutionLog({
      format: "json",
      tail: 5,
      event: "PreToolUse",
    });

    expect(Array.isArray(records)).toBe(true);
    expect(records[0]).toMatchObject({
      event: "PreToolUse",
      decision: expect.any(String),
    });
  });
});
