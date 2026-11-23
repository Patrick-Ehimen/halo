import { ConfirmDialog } from "@halo/ui";
import { useState } from "react";
import { tradingApi, type TradingView } from "../api";
import { useTradingStore } from "../store";

const CIRCUMFERENCE = 2 * Math.PI * 16;

/** The pre-trade checklist. Ticks reset every morning. */
export function Checklist({ view }: { view: TradingView }) {
  const run = useTradingStore((s) => s.run);
  const [draft, setDraft] = useState("");
  const [removing, setRemoving] = useState<{ id: string; label: string } | null>(null);
  const total = view.checklist.length;
  const fraction = total === 0 ? 0 : view.checklistDone / total;

  const add = async () => {
    if (await run(() => tradingApi.addItem(draft, ""))) setDraft("");
  };

  return (
    <section
      aria-label="Pre-trade checklist"
      className="flex min-w-0 flex-[1_1_300px] flex-col gap-1 rounded-2xl border border-card-border bg-card px-[18px] py-4"
    >
      <div className="mb-1.5 flex items-center gap-3">
        <svg width="40" height="40" viewBox="0 0 40 40" aria-hidden="true">
          <circle
            cx="20"
            cy="20"
            r="16"
            fill="none"
            stroke="rgba(255,255,255,0.08)"
            strokeWidth="4"
          />
          <circle
            cx="20"
            cy="20"
            r="16"
            fill="none"
            stroke="var(--halo-shield)"
            strokeWidth="4"
            strokeLinecap="round"
            strokeDasharray={`${fraction * CIRCUMFERENCE} ${CIRCUMFERENCE}`}
            transform="rotate(-90 20 20)"
            opacity={fraction > 0 ? 1 : 0}
          />
        </svg>
        <div className="flex flex-1 flex-col gap-0.5">
          <h2 className="text-[13px] font-semibold">Pre-trade checklist</h2>
          <span className="text-xs text-text-secondary">{view.checklistSummary}</span>
        </div>
        <button
          type="button"
          onClick={() => void run(tradingApi.resetChecklist)}
          disabled={view.checklistDone === 0}
          className="h-11 rounded-lg border border-white/10 px-3 text-xs text-text-secondary hover:text-text disabled:opacity-40"
        >
          Reset
        </button>
      </div>

      {total === 0 ? (
        <div className="flex flex-col items-start gap-2 rounded-xl border border-dashed border-white/16 bg-white/3 px-3.5 py-4">
          <p className="text-[13px] font-medium">No checklist yet</p>
          <p className="text-xs leading-normal text-text-secondary">
            Write the rules you check before every trade. Until you add some, only the head check
            and your window decide when you can trade.
          </p>
          <button
            type="button"
            onClick={() => void run(tradingApi.restoreSuggested)}
            className="mt-1 h-11 rounded-[10px] bg-white/10 px-3.5 text-[13px] font-medium text-text hover:bg-white/14"
          >
            Use suggested rules
          </button>
        </div>
      ) : (
        // A long checklist scrolls inside the card; the card stays its size.
        <div
          role="list"
          aria-label="Checklist rules"
          className="-mx-1 flex max-h-[340px] flex-col overflow-y-auto px-1"
        >
          {view.checklist.map((c) => (
            <div key={c.id} role="listitem" className="group flex items-center">
              <button
                type="button"
                aria-pressed={c.checked}
                onClick={() => void run(() => tradingApi.setChecked(c.id, !c.checked))}
                className="flex min-h-11 flex-1 items-center gap-2.5 rounded-[10px] px-2 py-1.5 text-left hover:bg-white/3"
              >
                <span
                  className={`flex size-5 shrink-0 items-center justify-center rounded-md ${
                    c.checked ? "bg-shield" : "border-[1.5px] border-white/28"
                  }`}
                >
                  {c.checked && (
                    <svg
                      width="12"
                      height="12"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="var(--halo-shield-ink)"
                      strokeWidth="3.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      aria-hidden="true"
                    >
                      <path d="M5 12l5 5 9-10" />
                    </svg>
                  )}
                </span>
                <span className="flex flex-1 flex-col gap-px">
                  <span
                    className={`text-[13px] font-medium ${c.checked ? "text-text-secondary" : "text-text"}`}
                  >
                    {c.label}
                  </span>
                  {c.hint && <span className="text-[11px] text-text-muted">{c.hint}</span>}
                </span>
              </button>
              <button
                type="button"
                aria-label={`Remove ${c.label}`}
                onClick={() => setRemoving({ id: c.id, label: c.label })}
                className="size-11 shrink-0 rounded-lg text-text-muted opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 hover:text-text"
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}

      <form
        className="mt-1.5 flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          void add();
        }}
      >
        <label htmlFor="newcheck" className="sr-only">
          Add a checklist item
        </label>
        <input
          id="newcheck"
          type="text"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder="Add your own rule"
          className="h-11 min-w-0 flex-1 rounded-[10px] border border-white/10 bg-white/4 px-3 text-[13px] text-text outline-none focus:border-white/25"
        />
        <button
          type="submit"
          disabled={!draft.trim()}
          className="h-11 rounded-[10px] bg-white/10 px-3.5 text-[13px] font-medium text-text disabled:opacity-40"
        >
          Add
        </button>
      </form>

      {removing && (
        <ConfirmDialog
          title="Remove this rule?"
          message={`“${removing.label}” comes off your pre-trade checklist for good, including today's tick.`}
          confirmLabel="Remove"
          onCancel={() => setRemoving(null)}
          onConfirm={() => {
            const { id } = removing;
            setRemoving(null);
            void run(() => tradingApi.removeItem(id));
          }}
        />
      )}
    </section>
  );
}
