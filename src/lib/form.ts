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

export async function formSubmit(snippetId: string, values: Record<string, string>): Promise<void> {
  return safeInvoke<void>("form_submit", { snippetId, values });
}

export async function formCancel(snippetId: string): Promise<void> {
  return safeInvoke<void>("form_cancel", { snippetId });
}
