import { HaloMark } from "@halo/ui";
import { useEffect, useState } from "react";
import { appApi } from "../../features/app/api";
import { subscribeTimer, timerApi, type CheckInResponse } from "../../features/focus/api";
import { useFocusStore } from "../../features/focus/store";

/** Unanswered check-ins close after this long. UI timing, not session time. */
const AUTO_DISMISS_MS = 2 * 60_000;

/** The "Still on it?" card, shown in the top-right corner during focus. */
export function CheckInWindow() {
  const { snapshot, connect } = useFocusStore();
  const [asked, setAsked] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => connect(), [connect]);

  useEffect(
    () =>
      subscribeTimer({
        onTick: () => {},
        onPhase: (e) => {
          if (e.type === "checkInDue") {
            setError(null);
            setAsked((n) => n + 1);
          }
        },
      }),
    [],
  );

  useEffect(() => {
    if (asked === 0) return;
    const t = setTimeout(() => void appApi.dismissCheckIn(), AUTO_DISMISS_MS);
    return () => clearTimeout(t);
  }, [asked]);

  const answer = async (response: CheckInResponse) => {
    try {
      await timerApi.checkIn(response);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="flex h-screen items-start gap-3 rounded-popover border border-white/10 bg-glass-popover p-4 text-text shadow-popover backdrop-blur-[40px] backdrop-saturate-150">
      <span className="flex size-10 shrink-0 items-center justify-center rounded-control bg-white/6">
        <HaloMark size={26} />
      </span>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-semibold">Still on it?</p>
        <p className="truncate text-[13px] text-text-secondary">
          {snapshot?.intention || "Your focus session"}
        </p>
        <div className="mt-3 flex gap-2">
          <button
            type="button"
            onClick={() => void answer("focused")}
            className="h-11 flex-1 rounded-control bg-focus text-[13px] font-semibold text-focus-ink"
          >
            Yes, focused
          </button>
          <button
            type="button"
            onClick={() => void answer("drifted")}
            className="h-11 flex-1 rounded-control border border-white/10 bg-control text-[13px] font-medium text-text hover:bg-white/9"
          >
            I drifted
          </button>
        </div>
        {error && <p className="mt-1.5 text-xs text-alert">{error}</p>}
      </div>
    </div>
  );
}
