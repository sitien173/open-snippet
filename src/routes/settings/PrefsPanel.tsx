import { useState, useEffect } from "react";
import { Prefs, getPrefs, listSnippets, setPrefs } from "../../lib/snippets";
import { ShellConsentDialog } from "./ShellConsentDialog";
import { I } from "../../lib/icons";

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
    return <div style={{ color: "var(--color-text-subdued)", padding: "16px" }}>Loading preferences...</div>;
  }

  if (!prefs) {
    return (
      <div className="warning-card" style={{ background: "var(--color-decorative-red)", borderColor: "var(--color-border-red)" }}>
        <div className="ico" style={{ color: "var(--color-text-red)" }}>
          <I.Warn />
        </div>
        <div className="body" style={{ color: "var(--color-text-red)" }}>
          <div className="title" style={{ color: "var(--color-text-red)", fontWeight: 600 }}>Failed to load preferences</div>
          <div>Please check console logs or restart the application.</div>
        </div>
      </div>
    );
  }

  return (
    <>
      <div data-testid="prefs-panel" className="prefs-panel" style={{ maxWidth: "660px" }}>
        <div className="toolbar" style={{ marginBottom: "24px" }}>
          <div className="toolbar-left">
            <h2>Preferences</h2>
          </div>
          <div className="toolbar-right">
            {saveStatus && (
              <span className={`badge ${saveStatus.includes("Error") ? "red" : saveStatus === "Saved" ? "green" : "gray"}`}>
                <span className="dot" />
                {saveStatus}
              </span>
            )}
          </div>
        </div>

        <div className="panel" style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "20px" }}>
          <div className="pref-item" style={{ display: "flex", alignItems: "flex-start", gap: "12px" }}>
            <input
              id="pref-paused"
              type="checkbox"
              checked={prefs.paused}
              onChange={(e) => handleChange("paused", e.target.checked)}
              style={{ marginTop: "4px", width: "16px", height: "16px", cursor: "pointer" }}
            />
            <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
              <label htmlFor="pref-paused" style={{ fontWeight: 500, cursor: "pointer" }}>Pause text expansion</label>
              <span style={{ fontSize: "12px", color: "var(--color-text-subdued)" }}>
                Temporarily disable all macro and snippet expansions.
              </span>
            </div>
          </div>

          <div className="pref-item" style={{ display: "flex", alignItems: "flex-start", gap: "12px", borderTop: "1px solid var(--color-border-subdued)", paddingTop: "16px" }}>
            <input
              id="pref-autostart"
              type="checkbox"
              checked={prefs.autostart}
              onChange={(e) => handleChange("autostart", e.target.checked)}
              style={{ marginTop: "4px", width: "16px", height: "16px", cursor: "pointer" }}
            />
            <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
              <label htmlFor="pref-autostart" style={{ fontWeight: 500, cursor: "pointer" }}>Start on system boot</label>
              <span style={{ fontSize: "12px", color: "var(--color-text-subdued)" }}>
                Automatically launch OpenMacro when logging into your computer.
              </span>
            </div>
          </div>

          <div className="field" style={{ borderTop: "1px solid var(--color-border-subdued)", paddingTop: "16px", marginBottom: 0 }}>
            <label htmlFor="pref-max-len" style={{ fontWeight: 500 }}>Max expansion length</label>
            <input
              id="pref-max-len"
              type="number"
              value={prefs.max_expansion_len}
              onChange={(e) => handleChange("max_expansion_len", parseInt(e.target.value, 10) || 0)}
              style={{ width: "100%", maxWidth: "200px", height: "36px", padding: "6px 10px" }}
            />
            <span className="help">
              The maximum characters a snippet is allowed to expand into (safety limit).
            </span>
          </div>

          <div className="pref-item" style={{ display: "flex", alignItems: "flex-start", gap: "12px", borderTop: "1px solid var(--color-border-subdued)", paddingTop: "16px" }}>
            <input
              id="pref-shell-consent"
              type="checkbox"
              checked={prefs.shell_consent}
              onChange={(e) => handleChange("shell_consent", e.target.checked)}
              style={{ marginTop: "4px", width: "16px", height: "16px", cursor: "pointer" }}
            />
            <div style={{ display: "flex", flexDirection: "column", gap: "4px", width: "100%" }}>
              <label htmlFor="pref-shell-consent" style={{ fontWeight: 500, cursor: "pointer" }}>Allow shell execution</label>
              <span style={{ fontSize: "12px", color: "var(--color-text-subdued)", marginBottom: "8px" }}>
                Enable running arbitrary terminal shell commands specified in snippet vars.
              </span>
              {prefs.shell_consent && (
                <div className="warning-card">
                  <div className="ico">
                    <I.Warn />
                  </div>
                  <div className="body">
                    <div className="title">Security alert</div>
                    <div>Shell execution allows snippets to run arbitrary terminal commands on your system. Ensure you trust all saved snippet sources.</div>
                  </div>
                </div>
              )}
            </div>
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
