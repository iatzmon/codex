import { requestHookValidation } from "../../ipc/hooks.js";

export interface ValidateOptions {
  scope?: "managed" | "project" | "local";
  format?: "text" | "json";
}

export interface ValidateSummary {
  status: "ok" | "warning" | "error";
  layers: Array<{
    scope: "managed" | "project" | "local";
    path: string;
    loadedHooks: number;
    checksum?: string;
  }>;
}

export async function validateHooks(
  options: ValidateOptions = {},
): Promise<ValidateSummary> {
  const summary = await requestHookValidation(options);
  if (options.format === "text") {
    renderValidationSummary(summary);
  }
  return summary;
}

function renderValidationSummary(summary: ValidateSummary): void {
  const status = summary.status.toUpperCase();
  console.log(`Validation status: ${status}`);
  if (summary.layers.length === 0) {
    console.log("No hook layers evaluated.");
  } else {
    summary.layers.forEach((layer) => {
      console.log(`- ${layer.scope} (${layer.path}) → hooks: ${layer.loadedHooks}`);
    });
  }
}
