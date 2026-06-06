import React, { useState, useEffect } from "react";
import { Snippet, VarDecl, saveSnippet, reloadSnippets } from "../../lib/snippets";
import { VarsPanel } from "./VarsPanel";

export interface SnippetEditorProps {
  snippet?: Snippet | null;
  allSnippets?: Snippet[];
  onSave?: () => void;
  onCancel?: () => void;
}

export function SnippetEditor({ snippet, allSnippets = [], onSave, onCancel }: SnippetEditorProps) {
  const [trigger, setTrigger] = useState("");
  const [replace, setReplace] = useState("");
  const [vars, setVars] = useState<VarDecl[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  // Sync state with snippet prop when it changes
  useEffect(() => {
    setTrigger(snippet?.trigger || "");
    setReplace(snippet?.replace || "");
    setVars(snippet?.vars || []);
    setError(null);
  }, [snippet]);

  const sourceFile = snippet?.source_file || "F:/projects_new/textblaze/openmacro/snippets/default.yaml";

  const isTriggerEmpty = !trigger.trim();
  const isTriggerTooLong = trigger.length > 32;
  const hasCollision = allSnippets.some(
    (s) =>
      s.trigger === trigger &&
      s.source_file === sourceFile &&
      s.id !== snippet?.id
  );

  const isSaveDisabled = isTriggerEmpty || isTriggerTooLong || hasCollision || isSaving;

  let validationError = "";
  if (isTriggerEmpty) {
    validationError = "Trigger cannot be empty";
  } else if (isTriggerTooLong) {
    validationError = "Trigger cannot exceed 32 characters";
  } else if (hasCollision) {
    validationError = "Trigger collision";
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
        trigger,
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
    <div data-testid="snippet-editor" className="snippet-editor">
      <h2>{snippet ? "Edit Snippet" : "New Snippet"}</h2>
      {error && <div className="error-message">{error}</div>}
      <form onSubmit={handleSave}>
        <div style={{ marginBottom: "1rem" }}>
          <label htmlFor="snippet-trigger">Trigger</label>
          <input
            id="snippet-trigger"
            type="text"
            value={trigger}
            onChange={(e) => setTrigger(e.target.value)}
          />
          {validationError && (
            <div className="validation-error" style={{ color: "red", marginTop: "0.25rem" }}>
              {validationError}
            </div>
          )}
        </div>

        <div style={{ marginBottom: "1rem" }}>
          <label htmlFor="snippet-replace">Replacement</label>
          <textarea
            id="snippet-replace"
            value={replace}
            onChange={(e) => setReplace(e.target.value)}
          />
        </div>

        <VarsPanel vars={vars} onChange={setVars} />

        <div style={{ marginTop: "1.5rem" }} className="editor-actions">
          <button type="button" onClick={onCancel}>
            Cancel
          </button>
          <button type="submit" disabled={isSaveDisabled}>
            Save
          </button>
        </div>
      </form>
    </div>
  );
}
