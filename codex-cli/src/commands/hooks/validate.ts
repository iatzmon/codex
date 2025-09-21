export interface ValidateOptions {
  scope?: "managed" | "project" | "local";
  format?: "text" | "json";
}

export interface ValidateSummary {
  status: "ok" | "warning" | "error";
  layers: unknown[];
}

export async function validateHooks(
  _options: ValidateOptions,
): Promise<ValidateSummary> {
  throw new Error("validateHooks not implemented");
}
