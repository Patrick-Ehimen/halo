type IntentionCardProps = {
  /** Editable before a session starts; read-only while it runs. */
  editable: boolean;
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
};

export function IntentionCard({ editable, value, onChange, onSubmit }: IntentionCardProps) {
  return (
    <div className="flex w-full max-w-[540px] items-center gap-3 rounded-card border border-card-border bg-card px-4 py-3.5">
      <svg
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="var(--halo-focus)"
        strokeWidth="1.8"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="9" />
        <circle cx="12" cy="12" r="5" />
        <circle cx="12" cy="12" r="1.5" fill="var(--halo-focus)" />
      </svg>
      <div className="min-w-0 flex-1">
        <label
          htmlFor="intention"
          className="block text-[11px] font-semibold tracking-[0.06em] text-text-muted uppercase"
        >
          Intention
        </label>
        {editable ? (
          <input
            id="intention"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") onSubmit();
            }}
            placeholder="What will you focus on?"
            maxLength={200}
            autoComplete="off"
            className="mt-0.5 w-full bg-transparent text-[15px] font-medium text-text outline-none placeholder:text-text-muted select-text"
          />
        ) : (
          <p id="intention" className="mt-0.5 truncate text-[15px] font-medium">
            {value || "No intention set"}
          </p>
        )}
      </div>
    </div>
  );
}
