import { commands, type AppInfo } from "@halo/core";

export type { AppInfo };

export async function getAppInfo(): Promise<AppInfo> {
  const result = await commands.appInfo();
  if (result.status === "error") {
    throw new Error(result.error.message);
  }
  return result.data;
}
