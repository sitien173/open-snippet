import { Prefs } from "../../lib/snippets";

type ShellConsentDialogProps = {
  prefs: Prefs;
  open: boolean;
  setPrefs: (prefs: Prefs) => Promise<void>;
  onClose: () => void;
};

export function ShellConsentDialog({
  prefs,
  open,
  setPrefs,
  onClose,
}: ShellConsentDialogProps) {
  if (!open || prefs.shell_consent) {
    return null;
  }

  const handleAccept = async () => {
    await setPrefs({ ...prefs, shell_consent: true });
    onClose();
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="shell-consent-title"
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(15, 23, 42, 0.72)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "1.5rem",
        zIndex: 1000,
      }}
    >
      <div
        style={{
          width: "min(100%, 32rem)",
          background: "#111827",
          color: "#e5e7eb",
          border: "1px solid #374151",
          borderRadius: "16px",
          boxShadow: "0 24px 80px rgba(15, 23, 42, 0.45)",
          padding: "1.5rem",
        }}
      >
        <h2 id="shell-consent-title" style={{ marginTop: 0, marginBottom: "0.75rem" }}>
          Shell execution consent
        </h2>
        <p style={{ marginTop: 0, lineHeight: 1.5 }}>
          Some snippets can execute local shell commands. This stays disabled until you accept it.
        </p>
        <div style={{ display: "flex", gap: "0.75rem", justifyContent: "flex-end", marginTop: "1.5rem" }}>
          <button type="button" onClick={onClose}>
            Decline
          </button>
          <button type="button" onClick={handleAccept}>
            Accept
          </button>
        </div>
      </div>
    </div>
  );
}
