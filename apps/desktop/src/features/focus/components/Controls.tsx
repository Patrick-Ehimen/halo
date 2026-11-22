import type { ReactNode } from "react";

type ControlsProps = {
  phase: "focus" | "shortBreak" | "longBreak";
  status: "idle" | "running" | "paused";
  canExtend: boolean;
  pausesLeft: number | null;
  onStartFocus: () => void;
  onStartBreak: () => void;
  onPause: () => void;
  onResume: () => void;
  onExtend: () => void;
  onEnd: () => void;
};

function Secondary({
  children,
  onClick,
  disabled,
}: {
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className="h-12 rounded-full border border-white/10 bg-control px-[18px] text-sm font-medium text-text transition-colors hover:bg-white/9 disabled:cursor-not-allowed disabled:opacity-40"
    >
      {children}
    </button>
  );
}

function Primary({
  children,
  onClick,
  accent,
  disabled,
  title,
}: {
  children: ReactNode;
  onClick: () => void;
  accent: "focus" | "break";
  disabled?: boolean;
  title?: string;
}) {
  const color =
    accent === "focus"
      ? "bg-focus text-focus-ink shadow-[0_10px_30px_rgba(242,181,68,0.28)]"
      : "bg-break text-break-ink shadow-[0_10px_30px_rgba(108,182,255,0.25)]";
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={`flex h-14 min-w-[168px] items-center justify-center gap-2.5 rounded-full px-[26px] text-[15px] font-semibold transition-transform active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50 ${color}`}
    >
      {children}
    </button>
  );
}

const PauseIcon = (
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.4"
    strokeLinecap="round"
    aria-hidden="true"
  >
    <path d="M8 5v14M16 5v14" />
  </svg>
);

const PlayIcon = (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
    <path d="M8 5l12 7-12 7z" />
  </svg>
);

export function Controls(p: ControlsProps) {
  const isBreak = p.phase !== "focus";

  if (p.status === "idle") {
    return (
      <div className="flex items-center gap-3">
        {isBreak && <Secondary onClick={p.onStartBreak}>Take break</Secondary>}
        <Primary accent="focus" onClick={p.onStartFocus}>
          {PlayIcon}
          {isBreak ? "Skip, focus again" : "Start focus"}
        </Primary>
      </div>
    );
  }

  const noPauses = !isBreak && p.pausesLeft === 0;
  return (
    <div className="flex items-center gap-3">
      {!isBreak && (
        <Secondary onClick={p.onExtend} disabled={!p.canExtend}>
          +5 min
        </Secondary>
      )}
      {p.status === "running" ? (
        <Primary
          accent={isBreak ? "break" : "focus"}
          onClick={p.onPause}
          disabled={noPauses}
          title={noPauses ? "No pauses left this session" : undefined}
        >
          {PauseIcon}
          Pause
        </Primary>
      ) : (
        <Primary accent={isBreak ? "break" : "focus"} onClick={p.onResume}>
          {PlayIcon}
          Resume
        </Primary>
      )}
      <Secondary onClick={p.onEnd}>{isBreak ? "Skip break" : "End"}</Secondary>
    </div>
  );
}
