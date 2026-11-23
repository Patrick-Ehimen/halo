import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";
import { statsApi, type ReflectionView } from "../../features/stats/api";

const RATINGS = [
  { value: 1, label: "Rough" },
  { value: 2, label: "Okay" },
  { value: 3, label: "Good" },
  { value: 4, label: "In flow" },
];

/** "How did it go?", shown at the top right after each focus session. */
export function ReflectionWindow() {
  const [view, setView] = useState<ReflectionView | null>(null);
  const [rating, setRating] = useState<number | null>(null);
  const [distractions, setDistractions] = useState(0);
  const [notes, setNotes] = useState("");
  const [error, setError] = useState<string | null>(null);

  const apply = useCallback((v: ReflectionView | null) => {
    setView(v);
    setRating(null);
    setDistractions(v?.distractions ?? 0);
    setNotes(v?.notes ?? "");
    setError(null);
  }, []);
  const fail = useCallback(
    (e: unknown) => setError(e instanceof Error ? e.message : String(e)),
    [],
  );

  // The card is reused for every session: reload whenever it's shown.
  useEffect(() => {
    statsApi.reflection().then(apply, fail);
    const pending = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) statsApi.reflection().then(apply, fail);
    });
    return () => void pending.then((unlisten) => unlisten());
  }, [apply, fail]);

  const save = async (startBreak: boolean) => {
    try {
      await statsApi.reflect({ rating, distractions, notes, startBreak });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="flex h-screen flex-col gap-4 rounded-[20px] border border-white/12 bg-glass-popover p-[22px] text-text backdrop-blur-[40px] backdrop-saturate-150">
      <div className="flex flex-col gap-1">
        <p className="text-xs font-semibold tracking-[0.06em] text-focus uppercase tabular-nums">
          Session complete{view ? ` · ${view.focusedLabel}` : ""}
        </p>
        <h1 className="text-xl font-semibold tracking-[-0.01em]">How did it go?</h1>
        {view?.intention && (
          <p className="truncate text-[13px] text-text-secondary">{view.intention}</p>
        )}
      </div>

      <div role="group" aria-label="Session rating" className="flex gap-1.5">
        {RATINGS.map((r) => {
          const on = rating === r.value;
          return (
            <button
              key={r.value}
              type="button"
              aria-pressed={on}
              onClick={() => setRating(on ? null : r.value)}
              className={`h-11 flex-1 rounded-[10px] border text-[13px] ${
                on
                  ? "border-focus bg-focus/16 font-semibold text-[#FFE2A6]"
                  : "border-white/10 bg-white/4 font-medium text-text-secondary hover:text-text"
              }`}
            >
              {r.label}
            </button>
          );
        })}
      </div>

      <div className="flex items-center justify-between text-[13px]">
        <span id="distractions-label" className="text-[#C9CBD5]">
          Distractions
        </span>
        <span className="flex items-center gap-2.5" aria-labelledby="distractions-label">
          <button
            type="button"
            aria-label="Fewer"
            disabled={distractions === 0}
            onClick={() => setDistractions((n) => Math.max(0, n - 1))}
            className="size-11 rounded-lg border border-white/12 text-base disabled:opacity-40"
          >
            −
          </button>
          <span className="min-w-5 text-center font-semibold tabular-nums" aria-live="polite">
            {distractions}
          </span>
          <button
            type="button"
            aria-label="More"
            onClick={() => setDistractions((n) => Math.min(999, n + 1))}
            className="size-11 rounded-lg border border-white/12 text-base"
          >
            +
          </button>
        </span>
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-1.5">
        <label htmlFor="notes" className="text-[13px] text-[#C9CBD5]">
          Session notes
        </label>
        <textarea
          id="notes"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          placeholder="What got done, what got in the way"
          className="min-h-0 flex-1 resize-none rounded-[10px] border border-white/10 bg-white/4 px-3 py-2.5 text-[13px] leading-normal text-text outline-none focus:border-white/25"
        />
      </div>

      {error && <p className="text-xs text-alert">{error}</p>}

      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => void save(false)}
          className="h-11 flex-1 rounded-control border border-white/12 text-[13px] font-medium text-text hover:bg-white/5"
        >
          {view?.breakLabel ? "Not now" : "Done"}
        </button>
        {view?.breakLabel && (
          <button
            type="button"
            onClick={() => void save(true)}
            className="h-11 flex-[1.4] rounded-control bg-break text-[13px] font-semibold text-break-ink"
          >
            {view.breakLabel}
          </button>
        )}
      </div>
    </div>
  );
}
