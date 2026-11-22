import { useEffect } from "react";
import { describe, presetOf } from "../../features/focus/format";
import { useFocusStore } from "../../features/focus/store";

const PauseIcon = (
  <svg
    width="14"
    height="14"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.6"
    strokeLinecap="round"
    aria-hidden="true"
  >
    <path d="M8 5v14M16 5v14" />
  </svg>
);

const PlayIcon = (
  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
    <path d="M8 5l12 7-12 7z" />
  </svg>
);

/** The design's mini ring: 24px, the Halo ring's proportions (r 22, stroke 7 on 64). */
function MiniRing({ progressPermille, accent }: { progressPermille: number; accent: string }) {
  const c = 2 * Math.PI * 22;
  const offset = c * (1 - Math.min(Math.max(progressPermille, 0), 1000) / 1000);
  return (
    <svg width="24" height="24" viewBox="0 0 64 64" aria-hidden="true" className="shrink-0">
      <circle cx="32" cy="32" r="22" fill="none" stroke="rgba(255,255,255,0.10)" strokeWidth="7" />
      <circle
        cx="32"
        cy="32"
        r="22"
        fill="none"
        stroke={accent}
        strokeWidth="7"
        strokeLinecap="round"
        strokeDasharray={c}
        strokeDashoffset={offset}
        transform="rotate(-90 32 32)"
        className="transition-[stroke-dashoffset] duration-500 ease-linear"
      />
    </svg>
  );
}

/**
 * The floating mini timer, as in the design: ring, time, intention, and a
 * play/pause button. Drag it anywhere; it stays on top. Hide it from the menu
 * bar menu, the popover, or ⌥⌘M.
 */
export function MiniWindow() {
  const { snapshot, connect } = useFocusStore();
  const actions = useFocusStore();

  useEffect(() => connect(), [connect]);

  if (!snapshot) {
    return (
      <div
        data-tauri-drag-region
        className="h-screen rounded-full border border-white/12 bg-surface"
      />
    );
  }

  const view = describe(snapshot);
  const idle = snapshot.status === "idle";
  const isBreak = snapshot.phase !== "focus";
  const running = snapshot.status === "running";
  const noPauses = running && !isBreak && snapshot.pausesLeft === 0;

  const label = isBreak ? view.label : snapshot.intention || (idle ? "Ready to focus" : view.label);

  const onPrimary = () => {
    if (running) void actions.pause();
    else if (snapshot.status === "paused") void actions.resume();
    else if (isBreak) void actions.startPending();
    else void actions.start(presetOf(snapshot), "");
  };
  const primaryLabel = running
    ? "Pause"
    : snapshot.status === "paused"
      ? "Resume"
      : isBreak
        ? "Start break"
        : "Start focus";

  return (
    <div
      data-tauri-drag-region
      className="flex h-screen items-center gap-3 rounded-full border border-white/12 bg-surface pr-2 pl-3.5 text-text"
    >
      <span data-tauri-drag-region className="flex">
        <MiniRing
          progressPermille={idle ? 0 : snapshot.progressPermille}
          accent={view.accent === "focus" ? "var(--halo-focus)" : "var(--halo-break)"}
        />
      </span>
      <span
        data-tauri-drag-region
        role="timer"
        className="shrink-0 text-[20px] leading-none tabular-nums"
      >
        {view.clock}
      </span>
      <span
        data-tauri-drag-region
        className="min-w-0 flex-1 truncate text-[13px] text-text-secondary"
      >
        {label}
      </span>
      <button
        type="button"
        aria-label={primaryLabel}
        title={noPauses ? "No pauses left this session" : primaryLabel}
        disabled={noPauses}
        onClick={onPrimary}
        className={`flex size-9 shrink-0 items-center justify-center rounded-full outline-none transition-colors focus-visible:ring-2 focus-visible:ring-focus/60 disabled:cursor-not-allowed disabled:opacity-40 ${
          isBreak ? "bg-break/18 text-break" : "bg-white/10 text-text hover:bg-white/14"
        }`}
      >
        {running ? PauseIcon : PlayIcon}
      </button>
    </div>
  );
}
