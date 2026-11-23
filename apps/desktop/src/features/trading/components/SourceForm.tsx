import { useState } from "react";
import type { ChartSource } from "@halo/core";

const fieldClass =
  "h-11 min-w-0 rounded-[10px] border border-white/10 bg-white/4 px-3 text-[13px] text-text outline-none focus:border-white/25";

const splitList = (s: string) =>
  s
    .split(",")
    .map((x) => x.trim())
    .filter(Boolean);

/** Edits one charting source: its name, apps and sites. */
export function SourceForm({
  initial,
  submitLabel,
  onSubmit,
  onCancel,
}: {
  initial: ChartSource;
  submitLabel: string;
  onSubmit: (source: ChartSource) => void;
  onCancel: () => void;
}) {
  const [label, setLabel] = useState(initial.label);
  const [apps, setApps] = useState(initial.apps.join(", "));
  const [sites, setSites] = useState(initial.hosts.join(", "));
  const id = `edit-${initial.label}`;
  const ready = label.trim() !== "" && (apps.trim() !== "" || sites.trim() !== "");

  return (
    <form
      aria-label={`Edit ${initial.label}`}
      className="flex flex-col gap-2 rounded-xl border border-white/8 bg-white/3 p-3"
      onSubmit={(e) => {
        e.preventDefault();
        if (!ready) return;
        onSubmit({ label: label.trim(), apps: splitList(apps), hosts: splitList(sites) });
      }}
    >
      <div className="grid grid-cols-[110px_1fr] items-center gap-x-3 gap-y-2">
        <label htmlFor={`${id}-label`} className="text-xs text-text-secondary">
          Name
        </label>
        <input
          id={`${id}-label`}
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          placeholder="Bybit"
          className={fieldClass}
        />
        <label htmlFor={`${id}-apps`} className="text-xs text-text-secondary">
          Apps
        </label>
        <input
          id={`${id}-apps`}
          value={apps}
          onChange={(e) => setApps(e.target.value)}
          placeholder="App names, like Bybit"
          className={fieldClass}
        />
        <label htmlFor={`${id}-sites`} className="text-xs text-text-secondary">
          Sites
        </label>
        <input
          id={`${id}-sites`}
          value={sites}
          onChange={(e) => setSites(e.target.value)}
          placeholder="Sites, like bybit.com"
          className={fieldClass}
        />
      </div>
      <p className="text-[11px] text-text-muted">
        Separate several with commas. A site counts its subdomains too.
      </p>
      <div className="flex justify-end gap-2">
        <button
          type="button"
          onClick={onCancel}
          className="h-11 rounded-[10px] px-3.5 text-[13px] font-medium text-text-secondary hover:text-text"
        >
          Cancel
        </button>
        <button
          type="submit"
          disabled={!ready}
          className="h-11 rounded-[10px] bg-white/10 px-3.5 text-[13px] font-medium text-text hover:bg-white/14 disabled:opacity-40"
        >
          {submitLabel}
        </button>
      </div>
    </form>
  );
}
