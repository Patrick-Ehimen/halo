import { useState } from "react";
import { tradingApi, type TradingView } from "../api";
import { useTradingStore } from "../store";

const WORDS = ["Tilted", "Off", "Okay", "Clear", "Sharp"];

function tone(n: number, on: boolean): string {
  if (!on) return "border-white/10 bg-white/4 text-text-secondary hover:text-text";
  if (n <= 2) return "border-alert bg-alert/18 text-[#FFD1CB]";
  if (n === 3) return "border-focus bg-focus/18 text-[#FFE2A6]";
  return "border-shield bg-shield/16 text-[#C6F2DA]";
}

/** "How clear is your head right now?", 1–5, plus what's going on. */
export function HeadCheck({
  view,
  onFocusBlock,
}: {
  view: TradingView;
  onFocusBlock?: () => void;
}) {
  const run = useTradingStore((s) => s.run);
  const logged = view.headCheck;
  const locked = logged?.lockedToday ?? false;
  const [factors, setFactors] = useState<string[]>(logged?.factors ?? []);

  // Follow what was logged (another window, or the next morning).
  const loggedKey = logged ? `${logged.score}:${logged.factors.join(",")}` : "";
  const [syncedKey, setSyncedKey] = useState(loggedKey);
  if (syncedKey !== loggedKey) {
    setSyncedKey(loggedKey);
    setFactors(logged?.factors ?? []);
  }

  const log = (score: number, next: string[]) => void run(() => tradingApi.headCheck(score, next));

  /** A second click on the picked score takes it back. */
  const clear = () => {
    setFactors([]);
    void run(tradingApi.clearHeadCheck);
  };

  const toggle = (f: string) => {
    const next = factors.includes(f) ? factors.filter((x) => x !== f) : [...factors, f];
    setFactors(next);
    if (logged && !locked) log(logged.score, next);
  };

  return (
    <section
      aria-label="Head check"
      className="flex min-w-0 flex-[1_1_300px] flex-col gap-3.5 rounded-2xl border border-card-border bg-card px-[18px] py-4"
    >
      <div className="flex flex-col gap-0.5">
        <h2 className="text-[13px] font-semibold text-focus">Head check</h2>
        <span className="text-xs text-text-secondary">
          {logged ? logged.atLabel : "How clear is your head right now?"}
        </span>
      </div>
      <div
        role="group"
        aria-label="Rate your state from 1 to 5"
        className="grid grid-cols-5 gap-1.5"
      >
        {WORDS.map((word, i) => {
          const n = i + 1;
          const on = logged?.score === n;
          return (
            <button
              key={n}
              type="button"
              aria-pressed={on}
              aria-label={`${n}, ${word}`}
              disabled={locked}
              onClick={() => (on ? clear() : log(n, factors))}
              className={`flex h-[60px] flex-col items-center justify-center gap-0.5 rounded-xl border disabled:cursor-not-allowed ${tone(n, on)} ${locked && !on ? "opacity-50" : ""}`}
            >
              <span className="text-lg font-semibold">{n}</span>
              <span className="text-[10px] font-medium">{word}</span>
            </button>
          );
        })}
      </div>
      <div className="flex flex-col gap-2">
        <span className="text-xs text-text-secondary">Anything else going on?</span>
        <div role="group" aria-label="Factors" className="flex flex-wrap gap-1.5">
          {view.factorOptions.map((f) => {
            const on = factors.includes(f);
            return (
              <button
                key={f}
                type="button"
                aria-pressed={on}
                disabled={locked}
                onClick={() => toggle(f)}
                className={`h-8 rounded-full border px-3 text-xs font-medium ${
                  on
                    ? "border-break/45 bg-break/16 text-[#CFE6FF]"
                    : "border-white/9 bg-white/4 text-text-secondary hover:text-text"
                }`}
              >
                {f}
              </button>
            );
          })}
        </div>
      </div>

      {!logged && (
        <p className="rounded-xl border border-dashed border-white/16 bg-white/4 px-3.5 py-3 text-[13px] leading-normal text-text-secondary">
          Rate yourself before your window opens. It takes five seconds and it's part of unlocking
          your window.
        </p>
      )}
      {logged && locked && (
        <div className="flex flex-col gap-2.5 rounded-xl border border-alert/34 bg-alert/10 px-3.5 py-3">
          <div className="flex flex-col gap-0.5">
            <span className="text-sm font-semibold text-[#FFD1CB]">Sit this session out</span>
            <span className="text-xs leading-normal text-[#FFD1CB]">
              Locked for the rest of today. Protecting the account is a win too.
            </span>
          </div>
          {onFocusBlock && (
            <button
              type="button"
              onClick={onFocusBlock}
              className="h-11 self-start rounded-[9px] bg-white/10 px-3 text-xs font-medium text-text"
            >
              Start a focus block instead
            </button>
          )}
        </div>
      )}
      {logged && logged.score === 3 && (
        <div className="flex flex-col gap-0.5 rounded-xl border border-focus/34 bg-focus/10 px-3.5 py-3">
          <span className="text-sm font-semibold text-[#FFE2A6]">Trade at half size</span>
          <span className="text-xs leading-normal text-[#FFE2A6]">
            You can trade, but cut size and cap it at 2 trades today.
          </span>
        </div>
      )}
      {logged && logged.score >= 4 && (
        <div className="flex flex-col gap-0.5 rounded-xl border border-shield/34 bg-shield/10 px-3.5 py-3">
          <span className="text-sm font-semibold text-[#C6F2DA]">Clear to trade</span>
          <span className="text-xs leading-normal text-[#C6F2DA]">
            Logged for today. Stick to your plan.
          </span>
        </div>
      )}
    </section>
  );
}
