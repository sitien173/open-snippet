import { useState, useEffect } from "react";
import { Prefs, getPrefs, listSnippets, setPrefs } from "../../lib/snippets";
import { ShellConsentDialog } from "./ShellConsentDialog";

export function PrefsPanel() {
  const [prefs, setPrefsState] = useState<Prefs | null>(null);
  const [loading, setLoading] = useState(true);
  const [saveStatus, setSaveStatus] = useState<string | null>(null);
  const [dialogDismissed, setDialogDismissed] = useState(false);
  const [checkedInitialConsent, setCheckedInitialConsent] = useState(false);
  const [showShellConsentDialog, setShowShellConsentDialog] = useState(false);

  useEffect(() => {
    getPrefs()
      .then((data) => {
        setPrefsState(data);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to load preferences", err);
        setLoading(false);
      });
  }, []);

  const persistPrefs = async (updated: Prefs) => {
    setPrefsState(updated);
    setSaveStatus("Saving...");
    try {
      await setPrefs(updated);
      setSaveStatus("Saved");
      setTimeout(() => setSaveStatus(null), 2000);
    } catch (err) {
      setSaveStatus("Error saving");
      console.error(err);
    }
  };

  useEffect(() => {
    if (!prefs || checkedInitialConsent) {
      return;
    }
    setCheckedInitialConsent(true);
    if (prefs.shell_consent) {
      return;
    }
    listSnippets()
      .then((snippets) => {
        if (snippets.some((snippet) => snippet.vars.some((variable) => variable.kind === "shell"))) {
          setShowShellConsentDialog(true);
        }
      })
      .catch((err) => {
        console.error("Failed to inspect snippets for shell consent", err);
      });
  }, [checkedInitialConsent, prefs]);

  const handleChange = async (key: keyof Prefs, value: boolean | number) => {
    if (!prefs) return;
    await persistPrefs({ ...prefs, [key]: value });
  };

  if (loading) {
    return <div className="panel-loading">Loading preferences...</div>;
  }

  if (!prefs) {
    return <div className="panel-error">Failed to load preferences.</div>;
  }

  return (
    <>
      <div data-testid="prefs-panel" className="prefs-panel">
        <div className="panel-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <h2>Preferences</h2>
          {saveStatus && <span className="save-status-tag">{saveStatus}</span>}
        </div>

        <div className="prefs-form" style={{ marginTop: "1rem" }}>
          <div className="pref-item" style={{ marginBottom: "1.5rem" }}>
            <label htmlFor="pref-paused" className="pref-label" style={{ display: "flex", alignItems: "center", gap: "0.5rem", cursor: "pointer" }}>
              <input
                id="pref-paused"
                type="checkbox"
                checked={prefs.paused}
                onChange={(e) => handleChange("paused", e.target.checked)}
              />
              <span>Pause text expansion</span>
            </label>
            <p className="pref-desc" style={{ margin: "0.25rem 0 0 1.5rem", fontSize: "0.85rem", opacity: 0.8 }}>
              Temporarily disable all macro and snippet expansions.
            </p>
          </div>

          <div className="pref-item" style={{ marginBottom: "1.5rem" }}>
            <label htmlFor="pref-autostart" className="pref-label" style={{ display: "flex", alignItems: "center", gap: "0.5rem", cursor: "pointer" }}>
              <input
                id="pref-autostart"
                type="checkbox"
                checked={prefs.autostart}
                onChange={(e) => handleChange("autostart", e.target.checked)}
              />
              <span>Start on system boot</span>
            </label>
            <p className="pref-desc" style={{ margin: "0.25rem 0 0 1.5rem", fontSize: "0.85rem", opacity: 0.8 }}>
              Automatically launch OpenMacro when logging into your computer.
            </p>
          </div>

          <div className="pref-item" style={{ marginBottom: "1.5rem" }}>
            <label htmlFor="pref-max-len" className="pref-label" style={{ display: "block", marginBottom: "0.25rem" }}>
              Max expansion length
            </label>
            <input
              id="pref-max-len"
              type="number"
              value={prefs.max_expansion_len}
              onChange={(e) => handleChange("max_expansion_len", parseInt(e.target.value, 10) || 0)}
              style={{ width: "100%", maxWidth: "300px" }}
            />
            <p className="pref-desc" style={{ margin: "0.25rem 0 0 0", fontSize: "0.85rem", opacity: 0.8 }}>
              The maximum characters a snippet is allowed to expand into (safety limit).
            </p>
          </div>

          <div className="pref-item" style={{ marginBottom: "1.5rem" }}>
            <label htmlFor="pref-shell-consent" className="pref-label" style={{ display: "flex", alignItems: "center", gap: "0.5rem", cursor: "pointer" }}>
              <input
                id="pref-shell-consent"
                type="checkbox"
                checked={prefs.shell_consent}
                onChange={(e) => handleChange("shell_consent", e.target.checked)}
              />
              <span>Allow shell execution</span>
            </label>
            <p className="pref-desc" style={{ margin: "0.25rem 0 0 1.5rem", fontSize: "0.85rem", opacity: 0.8 }}>
              Enable running arbitrary terminal shell commands specified in snippet vars.
            </p>
          </div>
        </div>
      </div>

      <ShellConsentDialog
        prefs={prefs}
        open={showShellConsentDialog && !dialogDismissed}
        setPrefs={persistPrefs}
        onClose={() => {
          setDialogDismissed(true);
          setShowShellConsentDialog(false);
        }}
      />
    </>
  );
}
