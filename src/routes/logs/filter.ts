export type RustLogEntry = {
  seq: number;
  ts_unix_ms: number;
  level: "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR";
  target: string;
  message: string;
  fields: Record<string, unknown>;
  span_path: string[];
};

export type FrontendLogEntry = {
  ts: number;
  level: "trace" | "debug" | "info" | "warn" | "error";
  module: string;
  msg: string;
  fields?: Record<string, unknown>;
};

export type UiEntry = {
  source: "rust" | "frontend";
  ts: number;
  level: "trace" | "debug" | "info" | "warn" | "error";
  module: string;
  msg: string;
  fields?: Record<string, unknown>;
  seq?: number;
  spanPath?: string[];
};

export type LogFilter = {
  source: "all" | "rust" | "frontend";
  minLevel: "trace" | "debug" | "info" | "warn" | "error";
  moduleQuery: string;
  searchQuery: string;
};

const LEVEL_PRIORITY: Record<UiEntry["level"], number> = {
  trace: 0,
  debug: 1,
  info: 2,
  warn: 3,
  error: 4,
};

export function mergeEntries(
  rustEntries: RustLogEntry[],
  frontEntries: FrontendLogEntry[]
): UiEntry[] {
  const normalizedRust: UiEntry[] = rustEntries.map(r => ({
    source: "rust",
    ts: r.ts_unix_ms,
    level: r.level.toLowerCase() as UiEntry["level"],
    module: r.target,
    msg: r.message,
    fields: r.fields,
    seq: r.seq,
    spanPath: r.span_path,
  }));

  const normalizedFront: UiEntry[] = frontEntries.map(f => ({
    source: "frontend",
    ts: f.ts,
    level: f.level,
    module: f.module,
    msg: f.msg,
    fields: f.fields,
  }));

  const combined = [...normalizedRust, ...normalizedFront].map((entry, idx) => ({
    entry,
    idx,
  }));

  combined.sort((a, b) => {
    if (a.entry.ts !== b.entry.ts) {
      return a.entry.ts - b.entry.ts;
    }
    return a.idx - b.idx;
  });

  return combined.map(c => c.entry);
}

export function applyFilter(rows: UiEntry[], filter: LogFilter): UiEntry[] {
  return rows.filter(entry => {
    if (filter.source !== "all" && entry.source !== filter.source) {
      return false;
    }

    const entryPriority = LEVEL_PRIORITY[entry.level] ?? 0;
    const filterPriority = LEVEL_PRIORITY[filter.minLevel] ?? 0;
    if (entryPriority < filterPriority) {
      return false;
    }

    if (filter.moduleQuery) {
      const modQuery = filter.moduleQuery.toLowerCase();
      if (!entry.module.toLowerCase().includes(modQuery)) {
        return false;
      }
    }

    if (filter.searchQuery) {
      const sQuery = filter.searchQuery.toLowerCase();
      const inMsg = entry.msg.toLowerCase().includes(sQuery);
      const inFields = entry.fields
        ? JSON.stringify(entry.fields).toLowerCase().includes(sQuery)
        : false;
      if (!inMsg && !inFields) {
        return false;
      }
    }

    return true;
  });
}
