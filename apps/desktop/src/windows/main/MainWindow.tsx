import { useEffect, useState } from "react";
import { getAppInfo } from "../../features/app/api";
import { FocusScreen } from "../../features/focus/FocusScreen";
import { Sidebar } from "./Sidebar";

export function MainWindow() {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getAppInfo()
      .then((info) => setVersion(info.version))
      .catch(() => setVersion(null));
  }, []);

  return (
    <div className="flex h-full bg-[radial-gradient(1100px_700px_at_10%_6%,rgba(242,181,68,0.14),transparent_60%),radial-gradient(1000px_700px_at_94%_96%,rgba(108,182,255,0.10),transparent_60%)]">
      <Sidebar version={version} />
      <main className="flex min-w-0 flex-1 flex-col">
        <FocusScreen />
      </main>
    </div>
  );
}
