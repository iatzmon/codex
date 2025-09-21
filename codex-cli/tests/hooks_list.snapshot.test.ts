import { describe, expect, it } from "vitest";

import { listHooks } from "../src/commands/hooks/list.js";

describe("codex hooks list snapshot", () => {
  it("matches the placeholder snapshot for JSON output", async () => {
    const result = await listHooks({ format: "json" });
    const normalized = {
      layers: result.layers.map((layer) => ({
        ...layer,
        path: sanitizePath(layer.path),
      })),
      events: result.events,
    };
    expect(normalized).toMatchFileSnapshot("./__snapshots__/hooks_list.snap.ts");
  });
});

function sanitizePath(value: string): string {
  const workspace = process.cwd();
  if (!value || !workspace) {
    return value;
  }
  return value.split(workspace).join("<workspace>");
}
