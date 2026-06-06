import { useState, useEffect } from "react";
import { LoadErrorDto, reloadSnippets, listLoadErrors } from "../../lib/snippets";

export function SyncPanel() {
  const [errors, setErrors] = useState<LoadErrorDto[]>([]);
  const [reloading, setReloading] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const fetchErrors = () => {
    listLoadErrors()
      .then(setErrors)
      .catch((err) => console.error("Failed to load snippet errors", err));
  };

  useEffect(() => {
    fetchErrors();
  }, []);

  const handleReload = async () => {
    setReloading(true);
    setMessage(null);
    try {
      const result = await reloadSnippets();
      setMessage(`Successfully reloaded! Loaded ${result.loaded} snippet(s).`);
      setErrors(result.errors);
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setMessage(`Reload failed: ${errMsg}`);
    } finally {
      setReloading(false);
    }
  };

  return (
    <div data-testid="sync-panel" className="sync-panel">
      <h2>Sync & Diagnostic Settings</h2>
      <p style={{ margin: "1rem 0" }}>
        Force a reload of the snippets directory and check for any syntax or parse errors in YAML files.
      </p>

      <button type="button" onClick={handleReload} disabled={reloading} style={{ marginBottom: "1.5rem" }}>
        {reloading ? "Reloading..." : "Reload Snippets"}
      </button>

      {message && (
        <div
          className="sync-message"
          style={{
            padding: "0.75rem",
            borderRadius: "6px",
            backgroundColor: "#2c2c2c",
            color: "#fff",
            marginBottom: "1.5rem",
          }}
        >
          {message}
        </div>
      )}

      <div className="errors-section">
        <h3>Snippet Load Errors ({errors.length})</h3>
        {errors.length === 0 ? (
          <p style={{ opacity: 0.8, color: "green" }}>✓ All snippets loaded without errors.</p>
        ) : (
          <div
            className="errors-list"
            style={{
              maxHeight: "300px",
              overflowY: "auto",
              border: "1px solid #444",
              borderRadius: "8px",
              marginTop: "0.5rem",
            }}
          >
            {errors.map((err, idx) => (
              <div
                key={idx}
                className="error-item"
                style={{
                  padding: "0.75rem",
                  borderBottom: idx < errors.length - 1 ? "1px solid #444" : "none",
                  backgroundColor: "#fee2e2",
                  color: "#991b1b",
                }}
              >
                <strong style={{ display: "block" }}>{err.path}</strong>
                <span>{err.message}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
