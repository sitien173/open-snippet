import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import { Snippet, listSnippets, getStoreSettings } from "../../lib/snippets";
import { SnippetList } from "./SnippetList";
import { SnippetEditor } from "./SnippetEditor";
import { PrefsPanel } from "./PrefsPanel";
import { SyncPanel } from "./SyncPanel";
import { I } from "../../lib/icons";

import { getLogger } from "../../lib/logger";

const log = getLogger("settings");

export function Settings() {
  const [activeTab, setActiveTab] = useState<"snippets" | "preferences" | "sync">("snippets");
  const [snippets, setSnippets] = useState<Snippet[]>([]);
  const [loading, setLoading] = useState(false);
  const [editingSnippet, setEditingSnippet] = useState<Snippet | null>(null);
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [triggerPrefix, setTriggerPrefix] = useState<string>(":");

  const fetchSnippets = async () => {
    setLoading(true);
    try {
      const data = await listSnippets();
      setSnippets(data);
    } catch (err) {
      log.error("Failed to load snippets", { error: err });
    } finally {
      setLoading(false);
    }
  };

  const fetchPrefix = async () => {
    try {
      const settings = await getStoreSettings();
      setTriggerPrefix(settings.trigger_prefix);
    } catch (err) {
      log.error("Failed to load trigger prefix settings", { error: err });
    }
  };

  useEffect(() => {
    fetchSnippets();
    fetchPrefix();
  }, []);

  const handleEditSnippet = (snippet: Snippet) => {
    setEditingSnippet(snippet);
    setIsEditorOpen(true);
  };

  const handleCreateSnippet = () => {
    setEditingSnippet(null);
    setIsEditorOpen(true);
  };

  const handleSaveEditor = () => {
    setIsEditorOpen(false);
    setEditingSnippet(null);
    fetchSnippets();
  };

  const handleCancelEditor = () => {
    setIsEditorOpen(false);
    setEditingSnippet(null);
  };

  return (
    <div className="shell">
      {/* Sidebar Navigation */}
      <aside className="sidebar">
        <div className="sidebar-brand">
          <h1>OpenMacro</h1>
          <span>Settings Console</span>
        </div>

        <nav style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
          <button
            type="button"
            className={`nav-item ${activeTab === "snippets" && !isEditorOpen ? "active" : ""}`}
            onClick={() => {
              setActiveTab("snippets");
              setIsEditorOpen(false);
            }}
          >
            <I.Scissors />
            Snippets
          </button>
          <button
            type="button"
            className={`nav-item ${activeTab === "preferences" ? "active" : ""}`}
            onClick={() => {
              setActiveTab("preferences");
              setIsEditorOpen(false);
            }}
          >
            <I.Settings />
            Preferences
          </button>
          <button
            type="button"
            className={`nav-item ${activeTab === "sync" ? "active" : ""}`}
            onClick={() => {
              setActiveTab("sync");
              setIsEditorOpen(false);
            }}
          >
            <I.RefreshCw />
            Sync & Diagnostics
          </button>
          <Link
            to="/logs"
            className="nav-item"
            style={{
              display: "flex",
              alignItems: "center",
              gap: "8px",
              textDecoration: "none",
              marginTop: "12px",
              borderTop: "1px solid var(--color-border-subdued)",
              paddingTop: "12px",
              color: "var(--color-text-subdued)",
            }}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" style={{ width: "18px", height: "18px" }}>
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" y1="19" x2="20" y2="19" />
            </svg>
            System Logs
          </Link>
        </nav>
      </aside>

      {/* Main Content Area */}
      <main className="main-content">
        {isEditorOpen ? (
          <SnippetEditor
            snippet={editingSnippet}
            allSnippets={snippets}
            onSave={handleSaveEditor}
            onCancel={handleCancelEditor}
            triggerPrefix={triggerPrefix}
          />
        ) : (
          <>
            {loading && <div style={{ color: "var(--color-text-subdued)", marginBottom: "1rem" }}>Loading snippets...</div>}
            {activeTab === "snippets" && (
              <SnippetList
                snippets={snippets}
                onEditSnippet={handleEditSnippet}
                onCreateSnippet={handleCreateSnippet}
              />
            )}
            {activeTab === "preferences" && (
              <PrefsPanel
                triggerPrefix={triggerPrefix}
                onPrefixSaved={(newPrefix) => setTriggerPrefix(newPrefix)}
              />
            )}
            {activeTab === "sync" && <SyncPanel />}
          </>
        )}
      </main>
    </div>
  );
}

// Keep SettingsRoute as an alias or main export to avoid breaking existing references
export function SettingsRoute() {
  return <Settings />;
}
