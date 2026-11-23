import { useEffect, useState, type ReactNode } from "react";
import type { QuickAddPreview } from "../api";

type QuickAddProps = {
  /** Asks Rust what the text means, for the chips. */
  preview: (text: string) => Promise<QuickAddPreview>;
  /** Adds the task. Resolves true when it was added. */
  onAdd: (text: string) => Promise<boolean>;
  placeholder?: string;
};

function Chip({ children, tone }: { children: ReactNode; tone: "sky" | "amber" | "plain" }) {
  const look =
    tone === "sky"
      ? "bg-break/15 text-break"
      : tone === "amber"
        ? "bg-focus/15 text-focus-soft"
        : "bg-white/6 text-text-secondary";
  return (
    <span
      className={`shrink-0 rounded-full px-2.5 py-1 text-xs font-medium whitespace-nowrap ${look}`}
    >
      {children}
    </span>
  );
}

/** "review PR tomorrow 10am ~2 sessions #meridian-rs", understood as you type. */
export function QuickAdd({ preview, onAdd, placeholder }: QuickAddProps) {
  const [text, setText] = useState("");
  const [seen, setSeen] = useState<QuickAddPreview | null>(null);

  // Ask for the chips shortly after typing stops (UI timing, not session time).
  useEffect(() => {
    if (!text.trim()) return;
    const t = setTimeout(() => {
      preview(text)
        .then(setSeen)
        .catch(() => setSeen(null));
    }, 120);
    return () => clearTimeout(t);
  }, [text, preview]);

  // Chips only while there's text; a stale preview never shows.
  const shown = text.trim() ? seen : null;

  const add = async () => {
    if (!text.trim()) return;
    if (await onAdd(text)) {
      setText("");
      setSeen(null);
    }
  };

  return (
    <div className="flex h-[52px] items-center gap-3 rounded-card border border-white/9 bg-white/5 pr-2 pl-4">
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="var(--halo-focus)"
        strokeWidth="1.9"
        strokeLinecap="round"
        aria-hidden="true"
      >
        <path d="M12 5v14M5 12h14" />
      </svg>
      <input
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            void add();
          }
        }}
        aria-label="Quick add a task"
        placeholder={placeholder ?? "Review order-book PR tomorrow 10am ~2 sessions #meridian-rs"}
        autoComplete="off"
        spellCheck={false}
        className="min-w-0 flex-1 bg-transparent text-[15px] text-text outline-none placeholder:text-text-muted select-text"
      />
      {shown && (
        <span className="flex min-w-0 items-center gap-1.5 overflow-hidden" aria-label="Understood">
          {(shown.dateLabel || shown.timeLabel) && (
            <Chip tone="sky">{[shown.dateLabel, shown.timeLabel].filter(Boolean).join(" ")}</Chip>
          )}
          {shown.estimate !== null && (
            <Chip tone="amber">
              {shown.estimate} {shown.estimate === 1 ? "session" : "sessions"}
            </Chip>
          )}
          {shown.recurrenceLabel && <Chip tone="plain">{shown.recurrenceLabel}</Chip>}
          {shown.projectName && (
            <Chip tone="plain">
              {shown.projectExists ? `#${shown.projectName}` : `New project: ${shown.projectName}`}
            </Chip>
          )}
          {shown.goalTitle && <Chip tone="amber">→ {shown.goalTitle}</Chip>}
        </span>
      )}
      <button
        type="button"
        onClick={() => void add()}
        disabled={!text.trim()}
        className="h-9 shrink-0 rounded-control border border-white/10 bg-control px-3.5 text-[13px] font-medium text-text hover:bg-white/9 disabled:opacity-40"
      >
        Add
      </button>
    </div>
  );
}
