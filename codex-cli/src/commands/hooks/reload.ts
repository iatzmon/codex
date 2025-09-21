import { requestHookReload } from "../../ipc/hooks.js";

export async function reloadHooks(): Promise<void> {
  const response = await requestHookReload();
  if (response.message) {
    console.log(response.message);
  }
}
