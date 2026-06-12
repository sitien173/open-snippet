import React, { useState } from "react";
import { UiEntry } from "./filter";

export interface LogRowProps {
  entry: UiEntry;
}

export const LogRow: React.FC<LogRowProps> = ({ entry }) => {
  const [expanded, setExpanded] = useState(false);

  const formatTime = (ts: number) => {
    try {
      return new Date(ts).toISOString();
    } catch {
      return new Date().toISOString();
    }
  };

  const levelColors: Record<UiEntry["level"], { bg: string; fg: string }> = {
    trace: { bg: "#4a4a4a", fg: "#ffffff" },
    debug: { bg: "#1f7a8c", fg: "#ffffff" },
    info: { bg: "#2d6a4f", fg: "#ffffff" },
    warn: { bg: "#ffb703", fg: "#121212" },
    error: { bg: "#d90429", fg: "#ffffff" },
  };

  const levelStyle = levelColors[entry.level] || { bg: "#808080", fg: "#ffffff" };

  const hasExtra =
    (entry.fields && Object.keys(entry.fields).length > 0) ||
    entry.seq !== undefined ||
    (entry.spanPath && entry.spanPath.length > 0);

  return (
    <div
      className={`log-row-container ${expanded ? "expanded" : ""}`}
      style={{
        borderBottom: "1px solid var(--color-border-subdued)",
        backgroundColor: expanded ? "var(--color-surface-muted)" : "var(--color-surface)",
        transition: "background-color var(--motion-fast) var(--ease-standard)",
        fontFamily: "var(--font-mono)",
        fontSize: "12px",
      }}
    >
      <div
        className="log-row-main"
        onClick={() => hasExtra && setExpanded(!expanded)}
        style={{
          display: "flex",
          alignItems: "center",
          padding: "6px 12px",
          cursor: hasExtra ? "pointer" : "default",
          gap: "8px",
        }}
      >
        {hasExtra ? (
          <span style={{ width: "12px", display: "inline-block", color: "var(--color-text-subdued)" }}>
            {expanded ? "▼" : "▶"}
          </span>
        ) : (
          <span style={{ width: "12px", display: "inline-block" }} />
        )}

        <span
          style={{
            fontSize: "10px",
            fontWeight: "bold",
            padding: "2px 4px",
            borderRadius: "3px",
            backgroundColor: entry.source === "rust" ? "var(--color-navigation-inverse)" : "var(--color-action-interactive)",
            color: "var(--color-text-inverse)",
            minWidth: "20px",
            textAlign: "center",
          }}
          title={entry.source === "rust" ? "Rust Subsystem" : "Frontend Logger"}
        >
          {entry.source === "rust" ? "R" : "F"}
        </span>

        <span style={{ color: "var(--color-text-subdued)", whiteSpace: "nowrap" }}>
          {formatTime(entry.ts)}
        </span>

        <span
          style={{
            backgroundColor: levelStyle.bg,
            color: levelStyle.fg,
            padding: "2px 6px",
            borderRadius: "4px",
            fontWeight: "bold",
            fontSize: "10px",
            textTransform: "uppercase",
            minWidth: "50px",
            textAlign: "center",
          }}
        >
          {entry.level}
        </span>

        <span
          style={{
            color: "var(--color-text-deep-green)",
            fontWeight: "bold",
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
            maxWidth: "180px",
          }}
          title={entry.module}
        >
          {entry.module}
        </span>

        <span style={{ color: "var(--color-border-subdued)" }}>|</span>

        <span
          style={{
            flex: 1,
            color: "var(--color-text-default)",
            wordBreak: "break-all",
          }}
        >
          {entry.msg}
        </span>
      </div>

      {expanded && hasExtra && (
        <div
          className="log-row-details"
          style={{
            padding: "8px 12px 12px 32px",
            backgroundColor: "rgba(0, 0, 0, 0.02)",
            borderTop: "1px dashed var(--color-border-subdued)",
          }}
        >
          {entry.seq !== undefined && (
            <div style={{ marginBottom: "4px" }}>
              <strong style={{ color: "var(--color-text-subdued)" }}>Sequence:</strong> {entry.seq}
            </div>
          )}
          {entry.spanPath && entry.spanPath.length > 0 && (
            <div style={{ marginBottom: "4px" }}>
              <strong style={{ color: "var(--color-text-subdued)" }}>Span Path:</strong>{" "}
              {entry.spanPath.join(" > ")}
            </div>
          )}
          {entry.fields && Object.keys(entry.fields).length > 0 && (
            <div>
              <strong style={{ color: "var(--color-text-subdued)" }}>Fields:</strong>
              <pre
                style={{
                  margin: "4px 0 0 0",
                  padding: "8px",
                  backgroundColor: "#ffffff",
                  border: "1px solid var(--color-border-subdued)",
                  borderRadius: "4px",
                  overflowX: "auto",
                  fontFamily: "var(--font-mono)",
                  fontSize: "11px",
                  color: "var(--color-text-default)",
                }}
              >
                {JSON.stringify(entry.fields, null, 2)}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
