export interface ListHooksOptions {
  event?: string;
  scope?: "managed" | "project" | "local";
  format?: "text" | "json";
}

export interface ListHooksResult {
  layers: unknown[];
  events: unknown[];
}

export async function listHooks(_options: ListHooksOptions): Promise<ListHooksResult> {
  throw new Error("listHooks not implemented");
}
