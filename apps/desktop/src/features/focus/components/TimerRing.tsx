const R = 150;
const C = 2 * Math.PI * R;

type TimerRingProps = {
  /** Ring fill in thousandths, from Rust. */
  progressPermille: number;
  accent: "focus" | "break";
  label: string;
  clock: string;
  sub: string;
};

export function TimerRing({ progressPermille, accent, label, clock, sub }: TimerRingProps) {
  const color = accent === "focus" ? "var(--halo-focus)" : "var(--halo-break)";
  const offset = C * (1 - Math.min(Math.max(progressPermille, 0), 1000) / 1000);
  return (
    <div className="relative size-[340px]">
      <svg width="340" height="340" viewBox="0 0 340 340" aria-hidden="true">
        <circle
          cx="170"
          cy="170"
          r={R}
          fill="none"
          stroke="rgba(255,255,255,0.06)"
          strokeWidth="12"
        />
        <circle
          cx="170"
          cy="170"
          r={R}
          fill="none"
          stroke={color}
          strokeOpacity="0.14"
          strokeWidth="28"
          strokeLinecap="round"
          strokeDasharray={C}
          strokeDashoffset={offset}
          transform="rotate(-90 170 170)"
          className="transition-[stroke-dashoffset] duration-500 ease-linear"
        />
        <circle
          cx="170"
          cy="170"
          r={R}
          fill="none"
          stroke={color}
          strokeWidth="12"
          strokeLinecap="round"
          strokeDasharray={C}
          strokeDashoffset={offset}
          transform="rotate(-90 170 170)"
          className="transition-[stroke-dashoffset] duration-500 ease-linear"
        />
      </svg>
      <div className="absolute inset-0 flex flex-col items-center justify-center gap-2">
        <span className="text-[13px] font-medium text-text-secondary">{label}</span>
        <span
          role="timer"
          className="text-[80px] leading-none font-light tracking-[-0.035em] tabular-nums"
        >
          {clock}
        </span>
        <span className="max-w-[220px] text-center text-[13px] leading-snug text-balance text-text-muted">
          {sub}
        </span>
      </div>
    </div>
  );
}
