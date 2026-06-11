import { useState, useEffect } from "react";
import { Prefs, StoreSettings, getPrefs, getStoreSettings, listSnippets, setPrefs, setStoreSettings } from "../../lib/snippets";
import { ShellConsentDialog } from "./ShellConsentDialog";
import { I } from "../../lib/icons";
import { getLogger } from "../../lib/logger";

const log = getLogger("settings.prefs");

export interface PrefsPanelProps {
  triggerPrefix?: string;
  onPrefixSaved?: (newPrefix: string) => void;
}

export function PrefsPanel({ triggerPrefix, onPrefixSaved }: PrefsPanelProps = {}) {
  const [prefs, setPrefsState] = useState<Prefs | null>(null);
  const [storeSettings, setStoreSettingsState] = useState<StoreSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saveStatus, setSaveStatus] = useState<string | null>(null);
  const [prefixSaveStatus, setPrefixSaveStatus] = useState<string | null>(null);
  const [dialogDismissed, setDialogDismissed] = useState(false);
  const [checkedInitialConsent, setCheckedInitialConsent] = useState(false);
  const [showShellConsentDialog, setShowShellConsentDialog] = useState(false);
  const [prefixInput, setPrefixInput] = useState("");
  const [prefixError, setPrefixError] = useState<string | null>(null);
  const [noticeDismissed, setNoticeDismissed] = useState(false);

  useEffect(() => {
    Promise.all([getPrefs(), getStoreSettings()])
      .then(([prefsData, storeSettingsData]) => {
        setPrefsState(prefsData);
        setStoreSettingsState(storeSettingsData);
        setPrefixInput(storeSettingsData.trigger_prefix);
        setLoading(false);
      })
      .catch((err) => {
        log.error("Failed to load preferences", { error: err });
        setLoading(false);
      });
  }, []);

  useEffect(() => {
    if (triggerPrefix !== undefined) {
      setPrefixInput(triggerPrefix);
    }
  }, [triggerPrefix]);

  const persistPrefs = async (updated: Prefs) => {
    setPrefsState(updated);
    setSaveStatus("Saving...");
    try {
      await setPrefs(updated);
      setSaveStatus("Saved");
      setTimeout(() => setSaveStatus(null), 2000);
    } catch (err) {
      setSaveStatus("Error saving");
      log.error("Failed to persist preferences", { error: err });
    }
  };

  const persistPrefix = async () => {
    if (!storeSettings) return;
    setPrefixSaveStatus("Saving...");
    setPrefixError(null);
    try {
      const updated: StoreSettings = {
        trigger_prefix: prefixInput,
        expand_mode: storeSettings.expand_mode || "manual",
      };
      await setStoreSettings(updated);
      setStoreSettingsState(updated);
      setPrefixSaveStatus("Saved");
      if (onPrefixSaved) {
        onPrefixSaved(prefixInput);
      }
      setTimeout(() => setPrefixSaveStatus(null), 2000);
    } catch (err: unknown) {
      const msg = typeof err === "string" ? err : err instanceof Error ? err.message : "Error saving prefix";
      setPrefixError(msg);
      setPrefixSaveStatus(null);
      // Rollback to actual state
      setPrefixInput(storeSettings.trigger_prefix);
      log.error("Failed to persist trigger prefix", { error: err });
    }
  };

  const handleExpandModeChange = async (mode: "manual" | "auto") => {
    if (!storeSettings) return;
    const updated: StoreSettings = {
      trigger_prefix: storeSettings.trigger_prefix,
      expand_mode: mode,
    };
    try {
      await setStoreSettings(updated);
      setStoreSettingsState(updated);
      setNoticeDismissed(true);
    } catch (err) {
      log.error("Failed to save expand mode", { error: err });
    }
  };

  const handleDismissMigrationNotice = async () => {
    if (!storeSettings) return;
    const updated: StoreSettings = {
      trigger_prefix: storeSettings.trigger_prefix,
      expand_mode: storeSettings.expand_mode || "manual",
    };
    try {
      await setStoreSettings(updated);
      setStoreSettingsState(updated);
      setNoticeDismissed(true);
    } catch (err) {
      log.error("Failed to dismiss migration notice", { error: err });
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
        log.error("Failed to inspect snippets for shell consent", { error: err });
      });
  }, [checkedInitialConsent, prefs]);

  const handleChange = async (key: keyof Prefs, value: boolean | number) => {
    if (!prefs) return;
    await persistPrefs({ ...prefs, [key]: value });
  };

  if (loading) {
    return <div style={{ color: "var(--color-text-subdued)", padding: "16px" }}>Loading preferences...</div>;
  }

  if (!prefs || !storeSettings) {
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

        {storeSettings.expand_mode_missing && !noticeDismissed && (
          <div className="warning-card" style={{ background: "var(--color-decorative-blue)", borderColor: "var(--color-border-blue)", marginBottom: "24px" }}>
            <div className="ico" style={{ color: "var(--color-icon-blue)" }}>
              <I.Info style={{ width: "20px", height: "20px" }} />
            </div>
            <div className="body" style={{ color: "var(--color-icon-blue)", display: "flex", justifyContent: "space-between", alignItems: "center", width: "100%" }}>
              <span>Snippets now expand on Tab/Enter. Change this in Settings.</span>
              <button
                type="button"
                aria-label="dismiss migration notice"
                onClick={handleDismissMigrationNotice}
                style={{
                  background: "transparent",
                  border: "none",
                  cursor: "pointer",
                  color: "var(--color-icon-blue)",
                  padding: "4px",
                  display: "flex",
                  alignItems: "center"
                }}
              >
                <I.X style={{ width: "16px", height: "16px" }} />
              </button>
            </div>
          </div>
        )}

        <div className="panel" style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "20px" }}>
          <div className="field" style={{ marginBottom: 0 }}>
            <label htmlFor="pref-trigger-prefix" style={{ fontWeight: 500 }}>Trigger prefix</label>
            <div style={{ display: "flex", alignItems: "center", gap: "12px", width: "100%", maxWidth: "400px" }}>
              <input
                id="pref-trigger-prefix"
                type="text"
                value={prefixInput}
                onChange={(e) => setPrefixInput(e.target.value)}
                style={{ width: "120px", height: "36px", padding: "6px 10px" }}
              />
              <button className="btn btn-secondary sm" type="button" onClick={persistPrefix}>Save prefix</button>
              {prefixSaveStatus && (
                <span className={`badge ${prefixSaveStatus === "Saved" ? "green" : "gray"}`}>
                  <span className="dot" />
                  {prefixSaveStatus}
                </span>
              )}
            </div>
            {prefixError && (
              <div style={{ color: "var(--color-text-red)", fontSize: "12px", marginTop: "8px" }}>
                {prefixError}
              </div>
            )}
            <span className="help">
              Global prefix required before all snippet triggers. Preview: <code>{prefixInput}email</code>
            </span>
          </div>

          <div className="pref-item" style={{ display: "flex", alignItems: "flex-start", gap: "12px", borderTop: "1px solid var(--color-border-subdued)", paddingTop: "16px" }}>
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

          <div className="field" style={{ borderTop: "1px solid var(--color-border-subdued)", paddingTop: "16px", marginBottom: 0 }}>
            <label style={{ fontWeight: 500 }}>Expand mode</label>
            <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "8px" }}>
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <input
                  id="expand-mode-manual"
                  type="radio"
                  name="expand_mode"
                  value="manual"
                  checked={storeSettings.expand_mode === "manual"}
                  onChange={() => handleExpandModeChange("manual")}
                  style={{ width: "16px", height: "16px", cursor: "pointer" }}
                />
                <label htmlFor="expand-mode-manual" style={{ cursor: "pointer" }}>Manual</label>
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <input
                  id="expand-mode-auto"
                  type="radio"
                  name="expand_mode"
                  value="auto"
                  checked={storeSettings.expand_mode === "auto"}
                  onChange={() => handleExpandModeChange("auto")}
                  style={{ width: "16px", height: "16px", cursor: "pointer" }}
                />
                <label htmlFor="expand-mode-auto" style={{ cursor: "pointer" }}>Auto</label>
              </div>
            </div>
            <span className="help">
              Controls when snippets expand. Manual mode requires pressing Tab or Enter.
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
