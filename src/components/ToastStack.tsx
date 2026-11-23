import { X } from "lucide-react";
import type { ToastItem } from "../store/usePortStore";

interface ToastStackProps {
  toasts: ToastItem[];
  onDismiss: (id: number) => void;
}

export function ToastStack({ toasts, onDismiss }: ToastStackProps) {
  if (toasts.length === 0) {
    return null;
  }

  const latest = toasts[toasts.length - 1];
  const dotColor =
    latest.kind === "success"
      ? "var(--success-color)"
      : latest.kind === "warning"
        ? "var(--warning-color)"
        : latest.kind === "error"
          ? "var(--destructive-bg-color)"
          : "var(--accent-bg-color)";

  return (
    <div
      role="status"
      aria-live="polite"
      className="fixed bottom-10 left-1/2 -translate-x-1/2 z-50 pointer-events-none"
    >
      <div className="pointer-events-auto flex items-center gap-2.5 pl-3.5 pr-2 py-1.5 rounded-full bg-[var(--toast-bg-color)] text-[var(--toast-fg-color)] shadow-lg border border-white/10 text-[12.5px]">
        <span
          className="w-2 h-2 rounded-full shrink-0"
          style={{ backgroundColor: dotColor }}
          aria-hidden="true"
        />
        <span className="max-w-md truncate">{latest.message}</span>
        <button
          type="button"
          onClick={() => onDismiss(latest.id)}
          aria-label="Dismiss notification"
          className="w-6 h-6 rounded-full hover:bg-white/10 flex items-center justify-center text-white/75 hover:text-white cursor-pointer"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}
