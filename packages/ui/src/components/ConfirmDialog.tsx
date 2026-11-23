import { useEffect, useRef } from "react";
import { useDismiss } from "../hooks/useDismiss";

type ConfirmDialogProps = {
  title: string;
  message: string;
  /** The destructive action, e.g. "Remove". */
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
};

/** Asks before something that can't be undone. Escape or a click outside cancels. */
export function ConfirmDialog({
  title,
  message,
  confirmLabel,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const ref = useRef<HTMLDivElement>(null);
  const cancelRef = useRef<HTMLButtonElement>(null);
  useDismiss(ref, onCancel);
  // Start on Cancel, so Enter never removes by accident.
  useEffect(() => cancelRef.current?.focus(), []);

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/50 p-6">
      <div
        ref={ref}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
        aria-describedby="confirm-message"
        className="w-[380px] rounded-popover border border-white/10 bg-surface p-5 shadow-popover"
      >
        <h2 id="confirm-title" className="text-base font-semibold">
          {title}
        </h2>
        <p id="confirm-message" className="mt-2 text-[13px] leading-normal text-text-secondary">
          {message}
        </p>
        <div className="mt-5 flex justify-end gap-2">
          <button
            ref={cancelRef}
            type="button"
            onClick={onCancel}
            className="h-11 rounded-full border border-white/10 bg-control px-5 text-sm font-medium text-text hover:bg-white/9"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className="h-11 rounded-full bg-alert px-5 text-sm font-semibold text-[#2B0A06] hover:brightness-110"
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
