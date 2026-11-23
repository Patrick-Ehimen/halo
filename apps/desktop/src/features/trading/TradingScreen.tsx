import { useEffect, useState } from "react";
import { ChartTime } from "./components/ChartTime";
import { Checklist } from "./components/Checklist";
import { Clocks } from "./components/Clocks";
import { DeskSettingsDialog } from "./components/DeskSettingsDialog";
import { HeadCheck } from "./components/HeadCheck";
import type { TradingView } from "./api";
import { useTradingStore } from "./store";

const GATE_LOOK: Record<TradingView["gate"]["state"], string> = {
  steps: "border border-white/12 bg-white/6 text-text",
  sitOut: "border border-alert/50 text-[#FFD1CB]",
  ready: "bg-shield text-shield-ink",
  open: "bg-shield text-shield-ink",
};

export function TradingScreen({ onFocusBlock }: { onFocusBlock?: () => void }) {
  const { view, settings, error, connect } = useTradingStore();
  useEffect(() => connect(), [connect]);
  /** Desk settings open, and whether at the charting apps and sites. */
  const [settingsOpen, setSettingsOpen] = useState<"all" | "sources" | null>(null);

  return (
    <>
      <header
        data-tauri-drag-region
        className="flex flex-wrap items-center gap-3 border-b border-hairline px-7 py-4"
      >
        <div data-tauri-drag-region className="flex flex-1 flex-col gap-0.5">
          <h1 className="text-lg font-semibold tracking-[-0.01em]">Trading desk</h1>
          <span className="text-xs text-text-muted">{view?.gate.detail ?? " "}</span>
        </div>
        {view && (
          <>
            <span className="flex items-center gap-2 rounded-full border border-white/8 bg-white/5 px-3 py-1.5 text-xs text-[#C9CBD5] tabular-nums">
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                aria-hidden="true"
              >
                <circle cx="12" cy="12" r="9" />
                <path d="M12 7v5l3 2" />
              </svg>
              {view.zoneLabel} {view.nowLabel}
            </span>
            <span
              role="status"
              aria-label={`Gate: ${view.gate.label}`}
              className={`flex h-11 items-center rounded-control px-4 text-[13px] font-semibold ${GATE_LOOK[view.gate.state]}`}
            >
              {view.gate.label}
            </span>
            <button
              type="button"
              aria-label="Desk settings"
              title="Desk settings"
              onClick={() => setSettingsOpen("all")}
              className="flex size-11 items-center justify-center rounded-control border border-white/10 bg-control text-text-secondary hover:bg-white/9 hover:text-text"
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.75"
                strokeLinecap="round"
                aria-hidden="true"
              >
                <path d="M4 7h10M18 7h2M4 17h4M12 17h8" />
                <circle cx="16" cy="7" r="2" />
                <circle cx="10" cy="17" r="2" />
              </svg>
            </button>
          </>
        )}
      </header>

      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-auto px-7 pt-5 pb-7">
        {error && (
          <p role="alert" className="text-[13px] text-alert">
            {error}
          </p>
        )}
        {view ? (
          <>
            <Clocks view={view} />
            <div className="flex flex-wrap gap-4">
              <Checklist view={view} />
              <HeadCheck view={view} onFocusBlock={onFocusBlock} />
            </div>
            <ChartTime
              view={view}
              settings={settings}
              onEditSources={() => setSettingsOpen("sources")}
            />
            {settings && settingsOpen && (
              <DeskSettingsDialog
                settings={settings}
                view={view}
                focusSources={settingsOpen === "sources"}
                onClose={() => setSettingsOpen(null)}
              />
            )}
          </>
        ) : (
          !error && <p className="sr-only">Loading</p>
        )}
      </div>
    </>
  );
}
