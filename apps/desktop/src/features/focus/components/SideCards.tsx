import type { TodayFocus } from "../api";
import { formatHoursMinutes } from "../format";

const card = "rounded-card border border-card-border bg-card p-4";

export function TodayCard({ today, onLog }: { today: TodayFocus | null; onLog: () => void }) {
  const secs = today?.focusedSecs ?? 0;
  const sessions = today?.sessions ?? 0;
  return (
    <section aria-label="Today" className={`${card} p-[18px]`}>
      <div className="flex items-center gap-4">
        <svg width="56" height="56" viewBox="0 0 64 64" aria-hidden="true">
          <circle
            cx="32"
            cy="32"
            r="22"
            fill="none"
            stroke="rgba(255,255,255,0.08)"
            strokeWidth="6"
          />
          <circle
            cx="32"
            cy="32"
            r="22"
            fill="none"
            stroke="var(--halo-focus)"
            strokeWidth="6"
            strokeLinecap="round"
            strokeDasharray="118 20.23"
            transform="rotate(-50 32 32)"
            opacity={sessions > 0 ? 1 : 0.35}
          />
        </svg>
        <div>
          <p className="text-[28px] leading-tight font-semibold tabular-nums">
            {formatHoursMinutes(secs)}
          </p>
          <p className="text-[13px] text-text-secondary">focused today</p>
          <p className="text-xs text-text-muted tabular-nums">
            {sessions} {sessions === 1 ? "session" : "sessions"} saved
          </p>
        </div>
      </div>
      <button
        type="button"
        onClick={onLog}
        className="mt-3 h-11 w-full rounded-control border border-white/8 bg-control text-[13px] font-medium text-text-secondary transition-colors hover:text-text"
      >
        Log a missed session
      </button>
    </section>
  );
}

function ComingSoon({ title, body }: { title: string; body: string }) {
  return (
    <section aria-label={title} className={card}>
      <h2 className="text-sm font-semibold">{title}</h2>
      <p className="mt-1.5 text-[13px] text-text-muted">{body}</p>
    </section>
  );
}

export function UpNextCard() {
  return (
    <ComingSoon title="Up next" body="Plan your day in sessions. Tasks arrive with Today plan." />
  );
}

export function ShieldCard() {
  return (
    <section aria-label="Shield" className={`${card} flex items-start gap-3`}>
      <svg
        width="22"
        height="22"
        viewBox="0 0 24 24"
        fill="none"
        stroke="var(--halo-text-muted)"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d="M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6z" />
      </svg>
      <div>
        <h2 className="text-sm font-semibold">Shield is off</h2>
        <p className="mt-0.5 text-[13px] text-text-muted">Site and app blocking comes later.</p>
      </div>
    </section>
  );
}

export function AmbienceCard() {
  return <ComingSoon title="Ambience" body="Rain, café, brown noise and lo-fi are on the way." />;
}
