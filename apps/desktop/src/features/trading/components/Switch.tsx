/** A labelled on/off switch, amber like every switch in Halo. */
export function Switch({
  label,
  hint,
  checked,
  onChange,
}: {
  label: string;
  hint?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex min-h-11 cursor-pointer items-center justify-between gap-4 py-1.5">
      <span className="flex flex-col gap-0.5">
        <span className="text-[13px] font-medium">{label}</span>
        {hint && <span className="text-xs text-text-muted">{hint}</span>}
      </span>
      <input
        type="checkbox"
        role="switch"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="peer sr-only"
      />
      <span
        aria-hidden="true"
        className="relative h-7 w-[46px] shrink-0 rounded-full bg-white/14 transition-colors peer-checked:bg-focus peer-focus-visible:outline-2 peer-focus-visible:outline-focus after:absolute after:top-[3px] after:left-[3px] after:size-[22px] after:rounded-full after:bg-white after:transition-transform peer-checked:after:translate-x-[18px]"
      />
    </label>
  );
}
