import { useDismiss } from "@halo/ui";
import { useEffect, useRef, useState } from "react";
import { hideSplash } from "../../splash";
import { appApi } from "../app/api";
import { Controls } from "./components/Controls";
import { IntentionCard } from "./components/IntentionCard";
import { ManualLogDialog } from "./components/ManualLogDialog";
import { ModeSwitcher } from "./components/ModeSwitcher";
import { RoundDots } from "./components/RoundDots";
import { SettingChips } from "./components/SettingChips";
import { SettingsPanel } from "./components/SettingsPanel";
import { AmbienceCard, ShieldCard, TodayCard, UpNextCard } from "./components/SideCards";
import { TimerRing } from "./components/TimerRing";
import { PRESETS, describe, presetOf } from "./format";
import { useFocusStore } from "./store";
import { planApi } from "../plan/api";
import { model } from "../plan/model";
import { usePlanStore } from "../plan/store";
import { timerApi } from "./api";

export function FocusScreen({ onOpenPlan = () => {} }: { onOpenPlan?: () => void }) {
  const { snapshot, today, settings, prefs, error, connect, clearError } = useFocusStore();
  const actions = useFocusStore();
  const [intention, setIntention] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const settingsRef = useRef<HTMLDivElement>(null);
  useDismiss(settingsRef, () => setSettingsOpen(false), settingsOpen);
  const [logOpen, setLogOpen] = useState(false);

  useEffect(() => connect(), [connect]);
  const plan = usePlanStore((st) => st.view);
  const connectPlan = usePlanStore((st) => st.connect);
  useEffect(() => connectPlan(), [connectPlan]);

  // The menu bar and the palette can ask for a panel here.
  useEffect(
    () =>
      appApi.onUiRequest((action) => {
        if (action === "openSettings") setSettingsOpen(true);
        else setLogOpen(true);
      }),
    [],
  );

  // In-app shortcuts: ⌘K palette, ⌘1/⌘2/⌘3 start a preset, ⌘, settings.
  const idleOnFocus = snapshot?.status === "idle" && snapshot.phase === "focus";
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.metaKey || e.altKey || e.ctrlKey) return;
      if (e.key === "k") {
        e.preventDefault();
        void appApi.togglePalette();
      } else if (e.key === ",") {
        e.preventDefault();
        setSettingsOpen(true);
      } else if (["1", "2", "3"].includes(e.key) && idleOnFocus) {
        const preset = PRESETS[Number(e.key) - 1];
        if (preset) {
          e.preventDefault();
          void actions.start(preset.id, intention);
          setIntention("");
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [actions, intention, idleOnFocus]);

  // The launch splash stays up until there's a timer to show (or an error).
  const ready = snapshot !== null || error !== null;
  useEffect(() => {
    if (ready) hideSplash();
  }, [ready]);

  useEffect(() => {
    if (!error) return;
    const t = setTimeout(clearError, 4000);
    return () => clearTimeout(t);
  }, [error, clearError]);

  if (!snapshot) {
    return (
      <div className="flex flex-1 items-center justify-center text-sm text-text-muted">
        {error ? `Timer unavailable: ${error}` : "Loading timer"}
      </div>
    );
  }

  const view = describe(snapshot);
  const upNext = plan
    ? (() => {
        const m = model(plan);
        return plan.today
          .filter((t) => !t.parentId && !t.completed)
          .slice(0, 3)
          .map((t) => ({
            id: t.id,
            title: t.title,
            meta: [t.whenLabel, m.projectOf(t)?.name, m.goalOf(t)?.title]
              .filter(Boolean)
              .join(" · "),
            color: m.colorOf(t),
          }));
      })()
    : [];
  const preset = presetOf(snapshot);
  const idle = snapshot.status === "idle";
  const inFocus = snapshot.phase === "focus";
  const startFocus = () => {
    void actions.start(preset, intention);
    setIntention("");
  };

  return (
    <>
      <header
        data-tauri-drag-region
        className="flex flex-wrap items-center gap-3 border-b border-hairline px-7 py-4"
      >
        <h1 data-tauri-drag-region className="flex-1 text-lg font-semibold tracking-[-0.01em]">
          Focus
        </h1>
        <ModeSwitcher
          value={preset}
          disabled={!idle || !inFocus}
          onChange={(p) => void actions.selectPreset(p)}
        />
        <div ref={settingsRef} className="relative">
          <button
            type="button"
            aria-label="Timer settings"
            aria-expanded={settingsOpen}
            disabled={!settings}
            onClick={() => setSettingsOpen((open) => !open)}
            className="flex size-11 items-center justify-center rounded-control border border-white/8 bg-control text-text-secondary transition-colors hover:text-text disabled:opacity-40"
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.8"
              strokeLinecap="round"
              aria-hidden="true"
            >
              <path d="M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12M20 18h0" />
              <circle cx="16" cy="6" r="2" />
              <circle cx="10" cy="12" r="2" />
              <circle cx="18" cy="18" r="2" />
            </svg>
          </button>
          {settingsOpen && settings && (
            <SettingsPanel
              settings={settings}
              onChange={(s) => void actions.saveSettings(s)}
              prefs={prefs}
              onPrefsChange={(p) => void actions.savePrefs(p)}
              onClose={() => setSettingsOpen(false)}
            />
          )}
        </div>
      </header>

      <div className="flex min-h-0 flex-1 flex-wrap overflow-auto">
        <section
          aria-label="Current session"
          className="flex min-w-0 flex-[999_1_440px] flex-col items-center justify-center gap-[22px] px-7 py-8"
        >
          {inFocus || idle ? (
            <IntentionCard
              editable={idle}
              value={idle ? intention : snapshot.intention}
              onChange={setIntention}
              onSubmit={startFocus}
            />
          ) : (
            <p className="text-[15px] text-text-secondary">Step away from the screen.</p>
          )}

          <TimerRing
            progressPermille={idle ? 0 : snapshot.progressPermille}
            accent={view.accent}
            label={idle && inFocus ? "Ready" : view.label}
            clock={view.clock}
            sub={view.sub}
          />

          <RoundDots
            completed={snapshot.completedInCycle}
            total={snapshot.longBreakEvery}
            inFocus={inFocus}
          />

          <Controls
            phase={snapshot.phase}
            status={snapshot.status}
            canExtend={snapshot.kind !== "flowtime"}
            pausesLeft={snapshot.pausesLeft}
            onStartFocus={startFocus}
            onStartBreak={() => void actions.startPending()}
            onPause={() => void actions.pause()}
            onResume={() => void actions.resume()}
            onExtend={() => void actions.extend(5)}
            onEnd={() => void actions.end()}
          />

          <SettingChips snapshot={snapshot} />

          {inFocus && !idle && (
            <label className="w-full max-w-[540px]">
              <span className="sr-only">Session note</span>
              <textarea
                key={snapshot.intention}
                defaultValue={snapshot.note}
                onBlur={(e) => void timerApi.setNote(e.target.value).catch(() => {})}
                rows={2}
                placeholder="Notes for this session"
                className="w-full resize-none rounded-card border border-card-border bg-card px-3.5 py-2.5 text-[13px] text-text placeholder:text-text-muted outline-none select-text focus:border-white/16"
              />
            </label>
          )}

          <p role="status" aria-live="polite" className="min-h-5 text-[13px] text-alert">
            {error}
          </p>
        </section>

        <aside className="flex flex-[1_1_300px] flex-col gap-3.5 border-l border-hairline px-6 py-7">
          <TodayCard today={today} onLog={() => setLogOpen(true)} />
          <UpNextCard
            tasks={upNext}
            canStart={idle && inFocus}
            onStart={(id) => void planApi.startTask(id, preset).catch(() => {})}
            onOpenPlan={onOpenPlan}
          />
          <ShieldCard />
          <AmbienceCard />
        </aside>
      </div>

      {logOpen && <ManualLogDialog onSave={actions.logSession} onClose={() => setLogOpen(false)} />}
    </>
  );
}
