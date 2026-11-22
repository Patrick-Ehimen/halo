import { useState, type FormEvent } from "react";
import type { SessionKind } from "../api";

type ManualLogDialogProps = {
  /** Resolves to an error message, or null once saved. */
  onSave: (
    kind: SessionKind,
    started: string,
    minutes: number,
    intention: string,
  ) => Promise<string | null>;
  onClose: () => void;
};

const KINDS: { value: SessionKind; label: string }[] = [
  { value: "pomodoro", label: "Pomodoro" },
  { value: "deep", label: "Deep" },
  { value: "flowtime", label: "Flowtime" },
  { value: "custom", label: "Custom" },
];

const field =
  "h-11 w-full rounded-control border border-white/10 bg-control px-3 text-sm text-text select-text";

export function ManualLogDialog({ onSave, onClose }: ManualLogDialogProps) {
  const [kind, setKind] = useState<SessionKind>("deep");
  const [started, setStarted] = useState("");
  const [minutes, setMinutes] = useState("50");
  const [intention, setIntention] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setSaving(true);
    const message = await onSave(kind, started, Number(minutes) || 0, intention);
    setSaving(false);
    if (message) setError(message);
    else onClose();
  };

  return (
    <div
      className="fixed inset-0 z-30 flex items-center justify-center bg-black/50"
      onKeyDown={(e) => {
        if (e.key === "Escape") onClose();
      }}
    >
      <form
        role="dialog"
        aria-label="Log a missed session"
        onSubmit={(e) => void submit(e)}
        className="w-[400px] rounded-popover border border-white/10 bg-glass-popover p-5 shadow-popover backdrop-blur-[40px]"
      >
        <h2 className="text-base font-semibold">Log a missed session</h2>
        <p className="mt-1 text-[13px] text-text-muted">
          For focus that happened away from the timer.
        </p>

        <div className="mt-4 grid gap-3">
          <label className="grid gap-1.5 text-[13px] font-medium">
            Kind
            <select
              value={kind}
              onChange={(e) => setKind(e.target.value as SessionKind)}
              className={field}
            >
              {KINDS.map((k) => (
                <option key={k.value} value={k.value}>
                  {k.label}
                </option>
              ))}
            </select>
          </label>
          <div className="grid grid-cols-[1fr_120px] gap-3">
            <label className="grid gap-1.5 text-[13px] font-medium">
              Started
              <input
                type="datetime-local"
                value={started}
                onChange={(e) => setStarted(e.target.value)}
                required
                className={field}
              />
            </label>
            <label className="grid gap-1.5 text-[13px] font-medium">
              Minutes
              <input
                type="number"
                min={1}
                max={600}
                value={minutes}
                onChange={(e) => setMinutes(e.target.value)}
                required
                className={`${field} tabular-nums`}
              />
            </label>
          </div>
          <label className="grid gap-1.5 text-[13px] font-medium">
            Intention
            <input
              value={intention}
              onChange={(e) => setIntention(e.target.value)}
              placeholder="What did you work on?"
              maxLength={200}
              className={field}
            />
          </label>
        </div>

        <p role="alert" className="mt-3 min-h-5 text-[13px] text-alert">
          {error}
        </p>

        <div className="mt-2 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="h-11 rounded-full border border-white/10 bg-control px-5 text-sm font-medium text-text"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={saving}
            className="h-11 rounded-full bg-focus px-5 text-sm font-semibold text-focus-ink disabled:opacity-50"
          >
            Log session
          </button>
        </div>
      </form>
    </div>
  );
}
