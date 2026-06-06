import { Prefs } from "../../lib/snippets";
import { I } from "../../lib/icons";

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
      className="scrim"
      role="dialog"
      aria-modal="true"
      aria-labelledby="shell-consent-title"
    >
      <div className="modal">
        <header>
          <h2 id="shell-consent-title">Shell Execution Consent</h2>
          <button
            type="button"
            className="icon-btn"
            onClick={onClose}
            aria-label="Close dialog"
            style={{ border: "none", background: "transparent", cursor: "pointer" }}
          >
            <I.X />
          </button>
        </header>

        <div className="body">
          <div className="warning-card">
            <div className="ico">
              <I.Warn />
            </div>
            <div className="body">
              <div className="title">Security alert</div>
              <div>Some snippets contain variables that execute local terminal shell commands. Allowing shell execution grants snippets full access to run command line instructions.</div>
            </div>
          </div>
          <p style={{ margin: 0, color: "var(--color-text-subdued)" }}>
            For your security, shell execution is disabled by default. If you decline, these variables will fail to expand.
          </p>
        </div>

        <footer>
          <button type="button" className="btn btn-secondary" onClick={onClose} style={{ marginRight: "12px" }}>
            Decline
          </button>
          <button type="button" className="btn primary" onClick={handleAccept}>
            Accept
          </button>
        </footer>
      </div>
    </div>
  );
}
