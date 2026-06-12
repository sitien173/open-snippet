import React, { useState, useEffect } from "react";
import { Snippet, VarDecl, saveSnippet, reloadSnippets, getStoreSettings } from "../../lib/snippets";
import { VarsPanel } from "./VarsPanel";
import { I } from "../../lib/icons";

export interface SnippetEditorProps {
  snippet?: Snippet | null;
  allSnippets?: Snippet[];
  onSave?: () => void;
  onCancel?: () => void;
  triggerPrefix?: string;
}

export function SnippetEditor({ snippet, allSnippets = [], onSave, onCancel, triggerPrefix }: SnippetEditorProps) {
  const [trigger, setTrigger] = useState("");
  const [triggerLiteral, setTriggerLiteral] = useState(false);
  const [replace, setReplace] = useState("");
  const [vars, setVars] = useState<VarDecl[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [localPrefix, setLocalPrefix] = useState<string>(":");

  // Sync state with snippet prop when it changes
  useEffect(() => {
    setTrigger(snippet?.trigger || "");
    setTriggerLiteral(snippet?.trigger_literal || false);
    setReplace(snippet?.replace || "");
    setVars(snippet?.vars || []);
    setError(null);
  }, [snippet]);

  useEffect(() => {
    if (triggerPrefix !== undefined) {
      setLocalPrefix(triggerPrefix);
    } else {
      getStoreSettings()
        .then((data) => {
          setLocalPrefix(data.trigger_prefix);
        })
        .catch(() => {
          // ignore or fallback to ":"
        });
    }
  }, [triggerPrefix]);

  const sourceFile =
    snippet?.source_file ||
    allSnippets.find((entry) => entry.file_relative === "default.yaml" || entry.file_relative.endsWith("/default.yaml"))?.source_file ||
    allSnippets[0]?.source_file ||
    "";

  const isTriggerEmpty = !trigger.trim();
  const isTriggerTooLong = trigger.length > 32;
  const currentEffectiveTrigger = triggerLiteral
    ? trigger
    : (trigger.startsWith(localPrefix) ? trigger : localPrefix + trigger);

  const hasCollision = allSnippets.some(
    (s) =>
      s.effective_trigger === currentEffectiveTrigger &&
      s.id !== snippet?.id
  );

  const isSourceFileMissing = !sourceFile;
  const isSaveDisabled = isTriggerEmpty || isTriggerTooLong || hasCollision || isSourceFileMissing || isSaving;

  let validationError = "";
  if (isTriggerEmpty) {
    validationError = "Trigger cannot be empty";
  } else if (isTriggerTooLong) {
    validationError = "Trigger cannot exceed 32 characters";
  } else if (hasCollision) {
    validationError = "Trigger collision with an existing snippet";
  } else if (isSourceFileMissing) {
    validationError = "No snippet file is available";
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (isSaveDisabled) return;

    setIsSaving(true);
    setError(null);

    try {
      await saveSnippet({
        source_file: sourceFile,
        original_trigger: snippet?.trigger || null,
        original_trigger_literal: snippet?.trigger_literal ?? null,
        trigger,
        trigger_literal: triggerLiteral,
        replace,
        vars,
      });
      await reloadSnippets();
      if (onSave) {
        onSave();
      }
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setError(errMsg);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div data-testid="snippet-editor" className="snippet-editor" style={{ maxWidth: "800px" }}>
      <div className="toolbar" style={{ marginBottom: "24px" }}>
        <div className="toolbar-left">
          <h2>{snippet ? "Edit Snippet" : "New Snippet"}</h2>
        </div>
      </div>

      {error && (
        <div className="warning-card" style={{ background: "var(--color-decorative-red)", borderColor: "var(--color-border-red)", marginBottom: "24px" }}>
          <div className="ico" style={{ color: "var(--color-text-red)" }}>
            <I.Warn />
          </div>
          <div className="body" style={{ color: "var(--color-text-red)" }}>
            <div className="title" style={{ color: "var(--color-text-red)", fontWeight: 600 }}>Failed to save snippet</div>
            <div>{error}</div>
          </div>
        </div>
      )}

      <form onSubmit={handleSave} className="panel" style={{ padding: "24px" }}>
        <div className={`field ${validationError ? "has-error" : ""}`}>
          <label htmlFor="snippet-trigger">Trigger</label>
          <div style={{ display: "flex", alignItems: "flex-start", gap: "12px", width: "100%", maxWidth: "400px" }}>
            <input
              id="snippet-trigger"
              type="text"
              value={trigger}
              onChange={(e) => setTrigger(e.target.value)}
              style={{ flex: 1 }}
            />
            <label style={{ display: "flex", alignItems: "center", gap: "6px", cursor: "pointer", fontSize: "14px", marginTop: "8px" }}>
              <input 
                type="checkbox" 
                checked={triggerLiteral} 
                onChange={(e) => setTriggerLiteral(e.target.checked)} 
                aria-label="Exact match"
              />
              Exact match
            </label>
          </div>
          {validationError ? (
            <div className="error-text">{validationError}</div>
          ) : (
            <div className="help">
              The text trigger that expands into this snippet (e.g. /greeting).
              {!triggerLiteral && trigger && (
                <span style={{ display: "block", marginTop: "4px" }}>
                  Effective trigger: <code style={{ background: "var(--color-bg-subdued)", padding: "2px 4px", borderRadius: "4px" }}>{trigger.startsWith(localPrefix) ? trigger : localPrefix + trigger}</code>
                </span>
              )}
            </div>
          )}
        </div>

        <div className="field">
          <label htmlFor="snippet-replace">Replacement</label>
          <textarea
            id="snippet-replace"
            value={replace}
            onChange={(e) => setReplace(e.target.value)}
            rows={6}
            style={{ width: "100%", resize: "vertical" }}
          />
          <div className="help">The text contents to expand into. Use variables below if dynamic fields are needed.</div>
        </div>

        <div style={{ marginTop: "24px", borderTop: "1px solid var(--color-border-subdued)", paddingTop: "24px" }}>
          <VarsPanel vars={vars} onChange={setVars} />
        </div>

        <div className="toolbar" style={{ marginTop: "32px", marginBottom: 0, borderTop: "1px solid var(--color-border-subdued)", paddingTop: "16px" }}>
          <div className="toolbar-left">
            <button type="button" className="btn btn-secondary" onClick={onCancel}>
              Cancel
            </button>
          </div>
          <div className="toolbar-right">
            <button type="submit" className="btn primary" disabled={isSaveDisabled}>
              {isSaving ? "Saving..." : "Save snippet"}
            </button>
          </div>
        </div>
      </form>
    </div>
  );
}
