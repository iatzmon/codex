import { fetchHookRegistrySnapshot } from "../../ipc/hooks.js";

export interface ListHooksOptions {
  event?: string;
  scope?: "managed" | "project" | "local";
  format?: "text" | "json";
}

export interface ListHooksResult {
  layers: Array<{
    scope: "managed" | "project" | "local";
    path: string;
    checksum: string;
    loadedHooks: number;
    skippedHooks: Array<{ hookId?: string; reason: string; details?: string }>;
  }>;
  events: Array<{
    event: string;
    hooks: Array<{
      id: string;
      scope: "managed" | "project" | "local";
      command: string[];
    }>;
  }>;
}

export async function listHooks(
  options: ListHooksOptions = {},
): Promise<ListHooksResult> {
  const snapshot = await fetchHookRegistrySnapshot(options);

  const filteredEvents = snapshot.events
    .map((entry) => {
      const hooks = entry.hooks.filter((hook) => {
        if (options.scope && hook.scope !== options.scope) {
          return false;
        }
        return true;
      });
      return { ...entry, hooks };
    })
    .filter((entry) => {
      if (options.event && entry.event !== options.event) {
        return false;
      }
      return entry.hooks.length > 0 || !options.scope;
    });

  const result: ListHooksResult = {
    layers: snapshot.layers.filter((layer) => {
      if (!options.scope) {
        return true;
      }
      return layer.scope === options.scope;
    }),
    events: filteredEvents,
  };

  if (options.format === "text") {
    renderHookList(result);
  }

  return result;
}

function renderHookList(result: ListHooksResult): void {
  if (result.events.length === 0) {
    console.log("No hooks are currently registered.");
    return;
  }

  const rows = result.events
    .flatMap((entry) =>
      entry.hooks.map((hook) => ({
        Event: entry.event,
        Scope: hook.scope,
        ID: hook.id,
        Command: hook.command.join(" "),
      })),
    )
    .sort((a, b) => a.Event.localeCompare(b.Event));

  console.table(rows);
}
