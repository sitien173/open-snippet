import React, { useEffect, useState, useRef, useCallback } from "react";
import { useParams } from "react-router-dom";
import { listSnippets, Snippet } from "../../lib/snippets";
import { formSubmit, formCancel } from "../../lib/form";
import { FieldRenderer } from "./FieldRenderer";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { getLogger } from "../../lib/logger";
import "./Form.css";

const log = getLogger("form");

export const FormRoute: React.FC = () => {
  const { snippetId } = useParams<{ snippetId: string }>();
  const [snippet, setSnippet] = useState<Snippet | null>(null);
  const [values, setValues] = useState<Record<string, string>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);

  const handleCancel = useCallback(() => {
    if (!snippetId) return;
    formCancel(snippetId)
      .catch((err) => log.error("Cancel IPC failed", { error: err }))
      .finally(() => {
        try {
          getCurrentWindow().close();
        } catch (err) {
          // Ignore
        }
      });
  }, [snippetId]);

  // Load snippet on mount
  useEffect(() => {
    let active = true;
    setLoading(true);
    listSnippets()
      .then((snippets) => {
        if (!active) return;
        const found = snippets.find((s) => s.id === snippetId);
        if (found) {
          setSnippet(found);
          // Initialize form values
          const initialValues: Record<string, string> = {};
          found.vars.forEach((v) => {
            if (["text", "textarea", "choice", "number"].includes(v.kind)) {
              let defaultValue = v.default || "";
              if (v.kind === "choice" && !v.default && v.options && v.options.length > 0) {
                defaultValue = v.options[0];
              }
              initialValues[v.name] = defaultValue;
            }
          });
          setValues(initialValues);
        } else {
          setErrorMsg(`Snippet "${snippetId}" not found.`);
        }
        setLoading(false);
      })
      .catch((err) => {
        if (!active) return;
        log.error("Failed to load snippets", { error: err });
        setErrorMsg("Failed to load snippet form.");
        setLoading(false);
      });

    return () => {
      active = false;
    };
  }, [snippetId]);

  // Handle auto-resizing of Tauri window using ResizeObserver
  useEffect(() => {
    if (!containerRef.current) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        try {
          const rect = entry.target.getBoundingClientRect();
          const ceilHeight = Math.ceil(rect.height);
          const appWindow = getCurrentWindow();
          if (appWindow && typeof appWindow.setSize === "function") {
            appWindow.setSize(new LogicalSize(400, ceilHeight));
          }
        } catch (err) {
          // Ignore if not running inside Tauri (e.g. testing)
        }
      }
    });

    observer.observe(containerRef.current);
    return () => {
      observer.disconnect();
    };
  }, [loading, snippet]);

  useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        handleCancel();
      }
    };
    window.addEventListener("keydown", handleGlobalKeyDown);
    return () => {
      window.removeEventListener("keydown", handleGlobalKeyDown);
    };
  }, [handleCancel]);

  const handleValueChange = (name: string, value: string) => {
    setValues((prev) => ({
      ...prev,
      [name]: value,
    }));
    // Clear error for this field if user types
    if (errors[name]) {
      setErrors((prev) => {
        const next = { ...prev };
        delete next[name];
        return next;
      });
    }
  };

  const handleSubmit = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!snippet || !snippetId) return;

    const newErrors: Record<string, string> = {};
    let firstInvalidName: string | null = null;

    const formVars = snippet.vars.filter((v) =>
      ["text", "textarea", "choice", "number"].includes(v.kind)
    );

    for (const v of formVars) {
      if (v.required) {
        const val = values[v.name] || "";
        if (!val.trim()) {
          newErrors[v.name] = `${v.label || v.name} is required.`;
          if (!firstInvalidName) {
            firstInvalidName = v.name;
          }
        }
      }
    }

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      if (firstInvalidName) {
        const inputEl = document.getElementById(`field-${firstInvalidName}`);
        if (inputEl) {
          inputEl.focus();
        }
      }
      return;
    }

    setErrors({});

    formSubmit(snippetId, values)
      .catch((err) => log.error("Submit IPC failed", { error: err }))
      .finally(() => {
        try {
          getCurrentWindow().close();
        } catch (err) {
          // Ignore
        }
      });
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      const target = e.target as HTMLElement;
      if (target.tagName.toLowerCase() === "textarea") {
        // Allow default multiline insert inside textareas
        return;
      }
      e.preventDefault();
      handleSubmit();
    }
  };

  if (loading) {
    return (
      <div className="form-root" ref={containerRef}>
        <div className="form-header">
          <h2 className="form-title">Loading...</h2>
        </div>
      </div>
    );
  }

  if (errorMsg || !snippet) {
    return (
      <div className="form-root" ref={containerRef}>
        <div className="form-header">
          <h2 className="form-title">Error</h2>
        </div>
        <div className="form-body">
          <div style={{ color: "red" }}>{errorMsg || "Form not found."}</div>
        </div>
        <div className="form-actions">
          <button className="btn btn-secondary" onClick={handleCancel}>
            Close
          </button>
        </div>
      </div>
    );
  }

  const formVars = snippet.vars.filter((v) =>
    ["text", "textarea", "choice", "number"].includes(v.kind)
  );

  // Find the first focusable field index
  const firstFocusableName = formVars[0]?.name;

  return (
    <div className="form-root" ref={containerRef}>
      <div className="form-header">
        <h2 className="form-title">Fill Snippet: {snippet.trigger}</h2>
      </div>
      <form onSubmit={handleSubmit} onKeyDown={handleKeyDown} noValidate className="form-body">
        {formVars.map((v) => (
          <FieldRenderer
            key={v.name}
            decl={v}
            value={values[v.name] ?? ""}
            onChange={(val) => handleValueChange(v.name, val)}
            autoFocus={v.name === firstFocusableName}
            error={errors[v.name]}
          />
        ))}
        <div className="form-actions">
          <button type="button" className="btn btn-secondary" onClick={handleCancel}>
            Cancel
          </button>
          <button type="submit" className="btn btn-primary">
            Submit
          </button>
        </div>
      </form>
    </div>
  );
};
