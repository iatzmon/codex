import { describe, expect, it } from "vitest";

import { listHooks } from "../src/commands/hooks/list.js";
import { reloadHooks } from "../src/commands/hooks/reload.js";
import { showExecutionLog } from "../src/commands/hooks/exec-log.js";
import { validateHooks } from "../src/commands/hooks/validate.js";

// Contract expectations derived from specs/003-add-a-hook/contracts/hooks-cli.md

describe("codex hooks cli contract", () => {
  it("lists hooks in JSON format", async () => {
    const result = await listHooks({ format: "json" });
    expect(result).toMatchObject({
      layers: expect.any(Array),
      events: expect.any(Array),
    });
  });

  it("validates hooks and returns summary", async () => {
    const summary = await validateHooks({ format: "json" });
    expect(summary).toMatchObject({
      status: expect.stringMatching(/^(ok|warning|error)$/),
      layers: expect.any(Array),
    });
  });

  it("reloads hooks without throwing", async () => {
    await expect(reloadHooks()).resolves.not.toThrow();
  });

  it("tails execution log", async () => {
    const records = await showExecutionLog({ format: "json", tail: 1 });
    expect(Array.isArray(records)).toBe(true);
    if (records.length > 0) {
      expect(records[0]).toMatchObject({
        event: expect.any(String),
        hookId: expect.any(String),
        decision: expect.objectContaining({ decision: expect.any(String) }),
      });
    }
  });
});
