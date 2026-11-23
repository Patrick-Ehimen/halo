type SmallRingProps = {
  size: number;
  /** Ring fill in thousandths, from Rust. */
  progressPermille: number;
  accent: "focus" | "break";
  stroke?: number;
};

/** A compact progress ring for the popover and mini timer. */
export function SmallRing({ size, progressPermille, accent, stroke = 4 }: SmallRingProps) {
  const r = (size - stroke) / 2;
  const c = 2 * Math.PI * r;
  const color = accent === "focus" ? "var(--halo-focus)" : "var(--halo-break)";
  // The track carries a hint of the accent, so an idle ring still has colour.
  const track = accent === "focus" ? "rgba(242,181,68,0.16)" : "rgba(108,182,255,0.16)";
  const offset = c * (1 - Math.min(Math.max(progressPermille, 0), 1000) / 1000);
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
      <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke={track} strokeWidth={stroke} />
      <circle
        cx={size / 2}
        cy={size / 2}
        r={r}
        fill="none"
        stroke={color}
        strokeWidth={stroke}
        strokeLinecap="round"
        strokeDasharray={c}
        strokeDashoffset={offset}
        transform={`rotate(-90 ${size / 2} ${size / 2})`}
        className="transition-[stroke-dashoffset] duration-500 ease-linear"
      />
    </svg>
  );
}
