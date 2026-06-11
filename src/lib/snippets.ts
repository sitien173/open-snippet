import { invoke } from "@tauri-apps/api/core";

export type VarKind = "text" | "textarea" | "choice" | "number" | "datetime" | "clipboard" | "cursor" | "shell" | "form";

export type VarDecl = {
  name: string;
  kind: VarKind;
  label?: string;
  default?: string;
  required?: boolean;
  options?: string[];
  format?: string;
};

export type Snippet = {
  id: string;
  trigger: string;
  effective_trigger: string;
  trigger_literal: boolean;
  replace: string;
  vars: VarDecl[];
  source_file: string;
  file_relative: string;
};

export type SaveSnippetDto = {
  source_file: string;
  original_trigger: string | null;
  original_trigger_literal?: boolean | null;
  trigger: string;
  trigger_literal?: boolean;
  replace: string;
  vars: VarDecl[];
};

export type StoreSettings = {
  trigger_prefix: string;
};

export type LoadErrorDto = {
  path: string;
  message: string;
};

export type Prefs = {
  paused: boolean;
  autostart: boolean;
  max_expansion_len: number;
  shell_consent: boolean;
};

export type ReloadResult = {
  loaded: number;
  errors: LoadErrorDto[];
};

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

export async function listSnippets(): Promise<Snippet[]> {
  return safeInvoke<Snippet[]>("list_snippets");
}

export async function listLoadErrors(): Promise<LoadErrorDto[]> {
  return safeInvoke<LoadErrorDto[]>("list_load_errors");
}

export async function saveSnippet(payload: SaveSnippetDto): Promise<void> {
  return safeInvoke<void>("save_snippet", { payload });
}

export async function reloadSnippets(): Promise<ReloadResult> {
  return safeInvoke<ReloadResult>("reload_snippets");
}

export async function getStoreSettings(): Promise<StoreSettings> {
  return safeInvoke<StoreSettings>("get_store_settings");
}

export async function setStoreSettings(settings: StoreSettings): Promise<void> {
  return safeInvoke<void>("set_store_settings", { settings });
}

export async function getPrefs(): Promise<Prefs> {
  return safeInvoke<Prefs>("get_prefs");
}

export async function setPrefs(prefs: Prefs): Promise<void> {
  return safeInvoke<void>("set_prefs", { prefs });
}
