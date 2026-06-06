import { useEffect, useState } from "react";

import { syncInit, SyncAuthMode, syncStatus, SyncStatus, syncTestConnection, syncTickNow, TickReport } from "../../lib/sync";

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
        console.error("Failed to load sync status", error);
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
    if (authChoice === "https" && !pat.trim()) {
      return "PAT is required for HTTPS.";
    }
    return null;
  };

  const handleTestConnection = async () => {
    const error = validate();
    if (error) {
      setInlineMessage(error);
      return;
    }

    try {
      await syncTestConnection(remote, auth(), authChoice === "https" ? pat : undefined);
      setInlineMessage("Connection OK.");
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

    try {
      await syncInit(remote, auth(), authChoice === "https" ? pat : undefined);
      setInlineMessage("Sync initialized.");
      setStatus(await syncStatus());
    } catch (err) {
      setInlineMessage(err instanceof Error ? err.message : String(err));
    }
  };

  const handleTickNow = async () => {
    try {
      const report = await syncTickNow();
      setTickMessage(formatTickReport(report));
      setStatus(await syncStatus());
    } catch (err) {
      setTickMessage(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <div data-testid="sync-panel" className="sync-panel">
      <h2>Git Sync</h2>
      <div style={{ display: "grid", gap: "1rem", maxWidth: "40rem" }}>
        <label htmlFor="sync-remote">
          Remote URL
          <input
            id="sync-remote"
            type="text"
            value={remote}
            onChange={(event) => setRemote(event.target.value)}
            style={{ display: "block", width: "100%", marginTop: "0.35rem" }}
          />
        </label>

        <fieldset>
          <legend>Auth</legend>
          <label htmlFor="sync-auth-https" style={{ marginRight: "1rem" }}>
            <input
              id="sync-auth-https"
              type="radio"
              name="sync-auth"
              checked={authChoice === "https"}
              onChange={() => setAuthChoice("https")}
            />
            HTTPS PAT
          </label>
          <label htmlFor="sync-auth-ssh">
            <input
              id="sync-auth-ssh"
              type="radio"
              name="sync-auth"
              checked={authChoice === "ssh"}
              onChange={() => setAuthChoice("ssh")}
            />
            SSH
          </label>
        </fieldset>

        {authChoice === "https" && (
          <>
            <label htmlFor="sync-username">
              Username
              <input
                id="sync-username"
                type="text"
                value={username}
                onChange={(event) => setUsername(event.target.value)}
                style={{ display: "block", width: "100%", marginTop: "0.35rem" }}
              />
            </label>
            <label htmlFor="sync-pat">
              Personal Access Token
              <input
                id="sync-pat"
                type="password"
                value={pat}
                onChange={(event) => setPat(event.target.value)}
                style={{ display: "block", width: "100%", marginTop: "0.35rem" }}
              />
            </label>
          </>
        )}

        <div style={{ display: "flex", gap: "0.75rem", flexWrap: "wrap" }}>
          <button type="button" onClick={handleTestConnection}>Test Connection</button>
          <button type="button" onClick={handleInit}>Save & Init</button>
          <button type="button" onClick={handleTickNow}>Sync Now</button>
        </div>

        {inlineMessage && <div>{inlineMessage}</div>}
        {tickMessage && <div>{tickMessage}</div>}

        <div>
          <strong>Status</strong>
          {loadingStatus ? (
            <div>Loading status...</div>
          ) : (
            <div>
              <div>Branch: {status.branch ?? "—"}</div>
              <div>Ahead/Behind: {status.ahead}/{status.behind}</div>
              <div>Last Tick: {status.last_tick_unix ?? "—"}</div>
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
