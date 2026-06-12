import { useEffect, useState } from "react";
import { syncInit, SyncAuthMode, syncStatus, SyncStatus, syncTestConnection, syncTickNow, TickReport } from "../../lib/sync";
import { I } from "../../lib/icons";

import { getLogger } from "../../lib/logger";

const log = getLogger("settings.sync");

type AuthChoice = "https" | "ssh";

const EMPTY_STATUS: SyncStatus = {
  branch: null,
  ahead: 0,
  behind: 0,
  last_tick_unix: null,
};

export function SyncPanel() {
  const [remote, setRemote] = useState("");
  const [authChoice, setAuthChoice] = useState<AuthChoice>("https");
  const [username, setUsername] = useState("git");
  const [pat, setPat] = useState("");
  const [status, setStatus] = useState<SyncStatus>(EMPTY_STATUS);
  const [inlineMessage, setInlineMessage] = useState<string | null>(null);
  const [tickMessage, setTickMessage] = useState<string | null>(null);
  const [loadingStatus, setLoadingStatus] = useState(true);

  useEffect(() => {
    syncStatus()
      .then((next) => {
        setStatus(next);
        setLoadingStatus(false);
      })
      .catch((error) => {
        log.error("Failed to load sync status", { error });
        setLoadingStatus(false);
      });
  }, []);

  const auth = (): SyncAuthMode =>
    authChoice === "ssh"
      ? { Ssh: null }
      : {
          HttpsPat: {
            host: hostFromRemote(remote),
            username,
          },
        };

  const validate = (): string | null => {
    if (!remote.trim()) {
      return "Remote URL is required.";
    }
    if (authChoice === "https") {
      if (!remote.trim().toLowerCase().startsWith("https://")) {
        return "HTTPS is required for this authentication mode.";
      }
      try {
        const url = new URL(remote.trim());
        if (!url.hostname) {
          return "Invalid HTTPS URL.";
        }
      } catch {
        return "Invalid HTTPS URL.";
      }
      if (!pat.trim()) {
        return "PAT is required for HTTPS.";
      }
    }
    return null;
  };

  const handleTestConnection = async () => {
    const error = validate();
    if (error) {
      setInlineMessage(error);
      return;
    }

    setInlineMessage("Testing connection...");
    try {
      await syncTestConnection(remote, auth(), authChoice === "https" ? pat : undefined);
      setInlineMessage("Connection OK");
    } catch (err) {
      setInlineMessage(err instanceof Error ? err.message : String(err));
    }
  };

  const handleInit = async () => {
    const error = validate();
    if (error) {
      setInlineMessage(error);
      return;
    }

    setInlineMessage("Initializing...");
    try {
      await syncInit(remote, auth(), authChoice === "https" ? pat : undefined);
      setInlineMessage("Sync initialized successfully");
      setStatus(await syncStatus());
    } catch (err) {
      setInlineMessage(err instanceof Error ? err.message : String(err));
    }
  };

  const handleTickNow = async () => {
    setTickMessage("Syncing...");
    try {
      const report = await syncTickNow();
      setTickMessage(formatTickReport(report));
      setStatus(await syncStatus());
    } catch (err) {
      setTickMessage(err instanceof Error ? err.message : String(err));
    }
  };

  const isMsgError = (msg: string | null) => {
    if (!msg) return false;
    const lower = msg.toLowerCase();
    return lower.includes("failed") || lower.includes("error") || lower.includes("required") || lower.includes("invalid") || lower.includes("could not");
  };

  const isMsgSuccess = (msg: string | null) => {
    if (!msg) return false;
    const lower = msg.toLowerCase();
    return lower.includes("ok") || lower.includes("success") || lower.includes("synced");
  };

  return (
    <div data-testid="sync-panel" className="sync-panel" style={{ maxWidth: "660px" }}>
      <div className="toolbar" style={{ marginBottom: "24px" }}>
        <div className="toolbar-left">
          <h2>Git Sync</h2>
        </div>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
        {/* Form Panel */}
        <div className="panel" style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "16px" }}>
          <div className="field">
            <label htmlFor="sync-remote">Remote URL</label>
            <input
              id="sync-remote"
              type="text"
              placeholder="e.g. https://github.com/user/snippets.git"
              value={remote}
              onChange={(event) => setRemote(event.target.value)}
              style={{ width: "100%" }}
            />
            <span className="help">The Git repository URL where your snippets are stored.</span>
          </div>

          <div className="field">
            <label>Authentication mode</label>
            <div style={{ display: "flex", gap: "24px", marginTop: "4px" }}>
              <label htmlFor="sync-auth-https" style={{ display: "flex", alignItems: "center", gap: "8px", fontWeight: "normal", cursor: "pointer" }}>
                <input
                  id="sync-auth-https"
                  type="radio"
                  name="sync-auth"
                  checked={authChoice === "https"}
                  onChange={() => setAuthChoice("https")}
                  style={{ width: "16px", height: "16px", cursor: "pointer" }}
                />
                HTTPS PAT
              </label>
              <label htmlFor="sync-auth-ssh" style={{ display: "flex", alignItems: "center", gap: "8px", fontWeight: "normal", cursor: "pointer" }}>
                <input
                  id="sync-auth-ssh"
                  type="radio"
                  name="sync-auth"
                  checked={authChoice === "ssh"}
                  onChange={() => setAuthChoice("ssh")}
                  style={{ width: "16px", height: "16px", cursor: "pointer" }}
                />
                SSH keys
              </label>
            </div>
          </div>

          {authChoice === "https" && (
            <>
              <div className="field">
                <label htmlFor="sync-username">Username</label>
                <input
                  id="sync-username"
                  type="text"
                  value={username}
                  onChange={(event) => setUsername(event.target.value)}
                  style={{ width: "100%", maxWidth: "300px" }}
                />
              </div>
              <div className="field">
                <label htmlFor="sync-pat">Personal access token</label>
                <input
                  id="sync-pat"
                  type="password"
                  value={pat}
                  onChange={(event) => setPat(event.target.value)}
                  style={{ width: "100%", maxWidth: "400px" }}
                />
              </div>
            </>
          )}

          <div className="toolbar" style={{ marginTop: "8px", marginBottom: 0 }}>
            <div className="toolbar-left">
              <button type="button" className="btn btn-secondary" onClick={handleTestConnection}>
                Test connection
              </button>
            </div>
            <div className="toolbar-right">
              <button type="button" className="btn primary" onClick={handleInit}>
                Save & initialize
              </button>
            </div>
          </div>

          {inlineMessage && (
            <div
              className={`badge ${isMsgError(inlineMessage) ? "red" : isMsgSuccess(inlineMessage) ? "green" : "gray"}`}
              style={{ alignSelf: "flex-start", marginTop: "8px" }}
            >
              <span className="dot" />
              {inlineMessage}
            </div>
          )}
        </div>

        {/* Sync Actions & Status Panel */}
        <div className="panel" style={{ padding: "24px" }}>
          <div className="toolbar" style={{ borderBottom: "1px solid var(--color-border-subdued)", paddingBottom: "12px", marginBottom: "16px" }}>
            <div className="toolbar-left">
              <h3 style={{ margin: 0 }}>Sync status</h3>
            </div>
            <div className="toolbar-right">
              <button type="button" className="btn btn-secondary sm" onClick={handleTickNow}>
                <I.RefreshCw style={{ width: 12, height: 12, marginRight: "4px" }} />
                Sync now
              </button>
            </div>
          </div>

          {tickMessage && (
            <div
              className={`badge ${isMsgError(tickMessage) ? "red" : isMsgSuccess(tickMessage) ? "green" : "gray"}`}
              style={{ marginBottom: "16px" }}
            >
              <span className="dot" />
              {tickMessage}
            </div>
          )}

          {loadingStatus ? (
            <div style={{ color: "var(--color-text-subdued)" }}>Loading status...</div>
          ) : (
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "16px" }}>
              <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                <span style={{ fontSize: "12px", color: "var(--color-text-placeholder)", textTransform: "uppercase", fontWeight: 500 }}>Active branch</span>
                <span className="mono" style={{ fontSize: "14px", fontWeight: 600, color: "var(--color-text-default)" }}>
                  {status.branch ?? "—"}
                </span>
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                <span style={{ fontSize: "12px", color: "var(--color-text-placeholder)", textTransform: "uppercase", fontWeight: 500 }}>Ahead / Behind</span>
                <span className="mono" style={{ fontSize: "14px", fontWeight: 600, color: "var(--color-text-default)" }}>
                  {status.ahead} / {status.behind}
                </span>
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                <span style={{ fontSize: "12px", color: "var(--color-text-placeholder)", textTransform: "uppercase", fontWeight: 500 }}>Last sync tick</span>
                <span className="mono" style={{ fontSize: "14px", fontWeight: 600, color: "var(--color-text-default)" }}>
                  {status.last_tick_unix ? new Date(status.last_tick_unix * 1000).toLocaleString() : "—"}
                </span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function hostFromRemote(remote: string): string {
  try {
    return new URL(remote).host || "unknown";
  } catch {
    return "unknown";
  }
}

function formatTickReport(report: TickReport): string {
  if (report.kind === "conflict" && report.dir) {
    return `Last result: conflict (${report.dir})`;
  }
  if (report.kind === "synced") {
    return "Last result: synced";
  }
  return "Last result: no changes";
}
