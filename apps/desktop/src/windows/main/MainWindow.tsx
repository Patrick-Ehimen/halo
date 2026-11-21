import { useEffect, useState } from "react";
import { HaloMark } from "@halo/ui";
import { getAppInfo, type AppInfo } from "../../features/app/api";

export function MainWindow() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getAppInfo()
      .then(setInfo)
      .catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)));
  }, []);

  return (
    <main className="flex h-full flex-col items-center justify-center gap-4">
      {/* Drag region under the overlay title bar */}
      <div data-tauri-drag-region className="fixed inset-x-0 top-0 h-10" />
      <HaloMark size={72} />
      <h1 className="text-2xl font-semibold">Halo</h1>
      <p className="text-sm text-text-secondary">More deep work, less drift.</p>
      <p className="text-xs text-text-muted tabular-nums">
        {error
          ? `Core unavailable: ${error}`
          : info
            ? `v${info.version} · schema ${info.schemaVersion}`
            : "Connecting to core"}
      </p>
    </main>
  );
}
