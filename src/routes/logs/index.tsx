import { useState, useEffect, useRef, useLayoutEffect } from "react";
import { Link } from "react-router-dom";
import { useVirtualizer } from "@tanstack/react-virtual";
import { invoke } from "@tauri-apps/api/core";
import { getRing } from "../../lib/logger";
import {
  mergeEntries,
  applyFilter,
  UiEntry,
  LogFilter,
  RustLogEntry,
  FrontendLogEntry,
} from "./filter";
import { LogRow } from "./LogRow";

async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (typeof window !== "undefined" && window.__OPENMACRO_MOCK_INVOKE) {
    return args !== undefined
      ? (window.__OPENMACRO_MOCK_INVOKE(cmd, args) as Promise<T>)
      : (window.__OPENMACRO_MOCK_INVOKE(cmd) as Promise<T>);
  }
  return invoke<T>(cmd, args);
}

export function LogsRoute() {
  const [rows, setRows] = useState<UiEntry[]>([]);
  const [isPaused, setIsPaused] = useState(false);
  const [filter, setFilter] = useState<LogFilter>({
    source: "all",
    minLevel: "trace",
    moduleQuery: "",
    searchQuery: "",
  });

  const lastSeq = useRef<number>(0);
  const seenFrontendLogs = useRef<Set<FrontendLogEntry>>(new Set());
  const parentRef = useRef<HTMLDivElement>(null);
  const shouldScrollRef = useRef<boolean>(true);

  // Poll Rust and Frontend logs
  useEffect(() => {
    if (isPaused) return;

    const poll = async () => {
      try {
        // 1. Fetch Rust logs
        let rustEntries: RustLogEntry[] = [];
        try {
          rustEntries = await safeInvoke<RustLogEntry[]>("get_log_ring", {
            sinceSeq: lastSeq.current,
          });
        } catch (err) {
          // Tauri not available or command failed - ignore silently
        }

        if (rustEntries.length > 0) {
          const maxSeq = Math.max(...rustEntries.map(r => r.seq));
          lastSeq.current = maxSeq;
        }

        // 2. Fetch Frontend logs and filter out already seen references
        const currentRing = getRing();
        const newFront = currentRing.filter(
          entry => !seenFrontendLogs.current.has(entry)
        );
        seenFrontendLogs.current = new Set(currentRing);

        if (rustEntries.length > 0 || newFront.length > 0) {
          const newMerged = mergeEntries(rustEntries, newFront);

          // Before updating state, check if user is at the bottom
          let wasAtBottom = true;
          if (parentRef.current) {
            const { scrollTop, clientHeight, scrollHeight } = parentRef.current;
            wasAtBottom = scrollTop + clientHeight >= scrollHeight - 8;
          }

          setRows(prev => {
            const combined = [...prev, ...newMerged];
            // Cap at 5000 rows
            if (combined.length > 5000) {
              return combined.slice(combined.length - 5000);
            }
            return combined;
          });

          if (wasAtBottom) {
            shouldScrollRef.current = true;
          }
        }
      } catch (err) {
        console.error("Error polling log rings:", err);
      }
    };

    poll();
    const interval = setInterval(poll, 500);

    return () => {
      clearInterval(interval);
    };
  }, [isPaused]);

  // Apply filters
  const filteredRows = applyFilter(rows, filter);

  // Virtualization
  const rowVirtualizer = useVirtualizer({
    count: filteredRows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 36,
    overscan: 10,
  });

  // Handle auto-scroll after state updates
  useLayoutEffect(() => {
    if (shouldScrollRef.current && parentRef.current) {
      parentRef.current.scrollTop = parentRef.current.scrollHeight;
      shouldScrollRef.current = false;
    }
  }, [rows]);

  const handleClear = () => {
    // Clear local state view only. This does NOT clear the underlying rings in memory
    // (the next poll will only bring in new entries after the current refs).
    setRows([]);
  };

  const handleCopy = () => {
    const text = filteredRows.map(e => JSON.stringify(e)).join("\n");
    navigator.clipboard.writeText(text);
  };

  const handleSave = () => {
    const text = filteredRows.map(e => JSON.stringify(e)).join("\n");
    const blob = new Blob([text], { type: "application/x-ndjson" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `openmacro-logs-${new Date().toISOString().replace(/[:.]/g, "-")}.jsonl`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="shell" style={{ display: "flex", flexDirection: "column", height: "100vh", padding: "24px", boxSizing: "border-box" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "16px", marginBottom: "20px" }}>
        <Link
          to="/settings"
          style={{
            textDecoration: "none",
            color: "var(--color-action-interactive)",
            fontSize: "14px",
            fontWeight: "bold",
            display: "flex",
            alignItems: "center",
            gap: "4px"
          }}
        >
          &larr; Settings
        </Link>
        <h1 className="ff-display-xs" style={{ margin: 0 }}>Log Console</h1>
      </div>

      {/* Filter Toolbar */}
      <div
        style={{
          display: "flex",
          gap: "12px",
          marginBottom: "12px",
          flexWrap: "wrap",
          padding: "16px",
          backgroundColor: "var(--color-surface)",
          borderRadius: "6px",
          border: "1px solid var(--color-border-subdued)",
          boxShadow: "var(--elev-button)"
        }}
      >
        <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
          <label htmlFor="log-source-select" style={{ fontSize: "12px", fontWeight: "bold", color: "var(--color-text-subdued)" }}>Source</label>
          <select
            id="log-source-select"
            value={filter.source}
            onChange={e => setFilter(f => ({ ...f, source: e.target.value as "all" | "rust" | "frontend" }))}
            style={{ minWidth: "120px" }}
          >
            <option value="all">All Sources</option>
            <option value="rust">Rust</option>
            <option value="frontend">Frontend</option>
          </select>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
          <label htmlFor="log-level-select" style={{ fontSize: "12px", fontWeight: "bold", color: "var(--color-text-subdued)" }}>Min Level</label>
          <select
            id="log-level-select"
            value={filter.minLevel}
            onChange={e => setFilter(f => ({ ...f, minLevel: e.target.value as "trace" | "debug" | "info" | "warn" | "error" }))}
            style={{ minWidth: "100px" }}
          >
            <option value="trace">TRACE</option>
            <option value="debug">DEBUG</option>
            <option value="info">INFO</option>
            <option value="warn">WARN</option>
            <option value="error">ERROR</option>
          </select>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: "4px", flex: 1, minWidth: "150px" }}>
          <label htmlFor="log-module-input" style={{ fontSize: "12px", fontWeight: "bold", color: "var(--color-text-subdued)" }}>Module</label>
          <input
            id="log-module-input"
            type="text"
            placeholder="Filter by module..."
            value={filter.moduleQuery}
            onChange={e => setFilter(f => ({ ...f, moduleQuery: e.target.value }))}
            style={{ width: "100%" }}
          />
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: "4px", flex: 2, minWidth: "200px" }}>
          <label htmlFor="log-search-input" style={{ fontSize: "12px", fontWeight: "bold", color: "var(--color-text-subdued)" }}>Search</label>
          <input
            id="log-search-input"
            type="text"
            placeholder="Search msg or fields..."
            value={filter.searchQuery}
            onChange={e => setFilter(f => ({ ...f, searchQuery: e.target.value }))}
            style={{ width: "100%" }}
          />
        </div>
      </div>

      {/* Action Toolbar */}
      <div style={{ display: "flex", gap: "8px", marginBottom: "16px", alignItems: "center" }}>
        <button
          onClick={() => setIsPaused(!isPaused)}
          className={isPaused ? "btn-primary" : "btn-secondary"}
          style={{ minWidth: "100px" }}
        >
          {isPaused ? "▶ Resume" : "⏸ Pause"}
        </button>
        <button onClick={handleClear} className="btn-destructive">
          🗑 Clear
        </button>
        <div style={{ flex: 1 }} />
        <span style={{ fontSize: "12px", color: "var(--color-text-subdued)", marginRight: "12px" }}>
          Showing {filteredRows.length} of {rows.length} logs
        </span>
        <button onClick={handleCopy} className="btn-secondary">
          📋 Copy
        </button>
        <button onClick={handleSave} className="btn-primary">
          💾 Save
        </button>
      </div>

      {/* Virtualized log list */}
      <div
        ref={parentRef}
        role="log"
        aria-live="polite"
        style={{
          flex: 1,
          overflowY: "auto",
          border: "1px solid var(--color-border-subdued)",
          borderRadius: "6px",
          backgroundColor: "var(--color-surface)",
          boxShadow: "inset 0 1px 3px rgba(0,0,0,0.05)"
        }}
      >
        {filteredRows.length === 0 ? (
          <div style={{ padding: "32px", textAlign: "center", color: "var(--color-text-subdued)", fontSize: "14px" }}>
            No log entries match the active filters.
          </div>
        ) : (
          <div
            style={{
              height: `${rowVirtualizer.getTotalSize()}px`,
              width: "100%",
              position: "relative",
            }}
          >
            {rowVirtualizer.getVirtualItems().map(virtualItem => {
              const entry = filteredRows[virtualItem.index];
              return (
                <div
                  key={virtualItem.key}
                  ref={rowVirtualizer.measureElement}
                  data-index={virtualItem.index}
                  style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    transform: `translateY(${virtualItem.start}px)`,
                  }}
                >
                  <LogRow entry={entry} />
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
export default LogsRoute;
