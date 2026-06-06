import { invoke } from "@tauri-apps/api/core";

export type SyncAuthMode =
  | { Ssh: null }
  | { HttpsPat: { host: string; username: string } };

export type SyncStatus = {
  branch: string | null;
  ahead: number;
  behind: number;
  last_tick_unix: number | null;
};

export type TickReport = {
  kind: string;
  dir: string | null;
  committed: boolean;
};

type MockInvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

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

export async function syncTestConnection(
  remote: string,
  auth: SyncAuthMode,
  pat?: string,
): Promise<void> {
  return safeInvoke<void>("sync_test_connection", { remote, auth, pat });
}

export async function syncInit(
  remote: string,
  auth: SyncAuthMode,
  pat?: string,
): Promise<void> {
  return safeInvoke<void>("sync_init", { remote, auth, pat });
}

export async function syncTickNow(): Promise<TickReport> {
  return safeInvoke<TickReport>("sync_tick_now");
}

export async function syncStatus(): Promise<SyncStatus> {
  return safeInvoke<SyncStatus>("sync_status");
}
