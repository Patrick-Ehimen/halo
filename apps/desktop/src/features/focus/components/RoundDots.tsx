type RoundDotsProps = {
  completed: number;
  total: number;
  /** Mark the session in progress (focus only). */
  inFocus: boolean;
};

export function RoundDots({ completed, total, inFocus }: RoundDotsProps) {
  const current = Math.min(completed + 1, total);
  const text = inFocus
    ? `Session ${current} of ${total} · long break after ${total}`
    : `${completed} of ${total} done · long break after ${total}`;
  return (
    <div className="flex items-center gap-2" aria-label={text}>
      {Array.from({ length: total }, (_, i) => {
        const cls =
          i < completed
            ? "bg-focus"
            : i === completed && inFocus
              ? "border-2 border-focus"
              : "bg-white/14";
        return <span key={i} className={`size-2.5 rounded-full ${cls}`} />;
      })}
      <span className="ml-1.5 text-xs text-text-muted">{text}</span>
    </div>
  );
}
