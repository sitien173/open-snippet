import log from "loglevel";
import prefix from "loglevel-plugin-prefix";
import { invoke } from "@tauri-apps/api/core";

// Interface for mock function
export type MockInvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

declare global {
  interface Window {
    __OPENMACRO_MOCK_INVOKE?: MockInvokeFn;
  }
}

async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (typeof window !== "undefined" && window.__OPENMACRO_MOCK_INVOKE) {
    return args !== undefined
      ? (window.__OPENMACRO_MOCK_INVOKE(cmd, args) as Promise<T>)
      : (window.__OPENMACRO_MOCK_INVOKE(cmd) as Promise<T>);
  }
  return invoke<T>(cmd, args);
}

export type LogEntry = {
  ts: number;                                // Date.now()
  level: "trace" | "debug" | "info" | "warn" | "error";
  module: string;
  msg: string;
  fields?: Record<string, unknown>;
};

export interface FrontendLogCfg {
  level: string;
  modules: Record<string, string>;
}

const RING_CAP = 1000;
const ring: LogEntry[] = [];
let verboseContent = false;

const SENSITIVE_KEYS = /^(body|content|clipboard|value|token|password|secret)$/i;

export function redact(fields: Record<string, unknown>): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(fields)) {
    if (SENSITIVE_KEYS.test(k) && typeof v === "string") {
      out[k] = `<redacted len=${[...v].length}>`;
    } else if (SENSITIVE_KEYS.test(k)) {
      out[k] = "<redacted>";
    } else {
      out[k] = v;
    }
  }
  return out;
}

prefix.reg(log);

const origFactory = log.methodFactory;
log.methodFactory = (methodName, logLevel, loggerName) => {
  const raw = origFactory(methodName, logLevel, loggerName);
  return (msg: string, fields?: Record<string, unknown>) => {
    const entry: LogEntry = {
      ts: Date.now(),
      level: methodName as LogEntry["level"],
      module: String(loggerName ?? "app"),
      msg,
    };
    if (fields !== undefined) {
      entry.fields = verboseContent ? fields : redact(fields);
    }
    ring.push(entry);
    if (ring.length > RING_CAP) {
      ring.shift();
    }
    raw(msg, entry.fields ?? "");
  };
};

prefix.apply(log, {
  format: (level, name, ts) =>
    `[${ts}] ${level.toUpperCase()} ${name ?? "app"}:`,
  timestampFormatter: (date) => date.toISOString(),
});

// Apply the method factory to existing loggers
log.setLevel(log.getLevel());

export function getLogger(module: string) {
  return log.getLogger(module);
}

export function getRing(): LogEntry[] {
  return ring.slice();
}

export async function initFromConfig() {
  let cfg: FrontendLogCfg = { level: "info", modules: {} };
  try {
    cfg = await safeInvoke<FrontendLogCfg>("get_logging_frontend_cfg");
  } catch {
    // Tauri not available (e.g. pure web dev) or command failed — keep defaults.
  }
  verboseContent = import.meta.env.VITE_LOG_VERBOSE === "1";
  const envLevel = import.meta.env.VITE_LOG_LEVEL as log.LogLevelDesc | undefined;
  log.setLevel((envLevel || cfg.level) as log.LogLevelDesc);
  for (const [mod, lvl] of Object.entries(cfg.modules)) {
    log.getLogger(mod).setLevel(lvl as log.LogLevelDesc);
  }
}
