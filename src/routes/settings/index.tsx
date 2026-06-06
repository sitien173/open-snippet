import { useState, useEffect } from "react";
import { Snippet, listSnippets } from "../../lib/snippets";
import { SnippetList } from "./SnippetList";
import { SnippetEditor } from "./SnippetEditor";
import { PrefsPanel } from "./PrefsPanel";
import { SyncPanel } from "./SyncPanel";
import { I } from "../../lib/icons";

export function Settings() {
  const [activeTab, setActiveTab] = useState<"snippets" | "preferences" | "sync">("snippets");
  const [snippets, setSnippets] = useState<Snippet[]>([]);
  const [loading, setLoading] = useState(false);
  const [editingSnippet, setEditingSnippet] = useState<Snippet | null>(null);
  const [isEditorOpen, setIsEditorOpen] = useState(false);

  const fetchSnippets = async () => {
    setLoading(true);
    try {
      const data = await listSnippets();
      setSnippets(data);
    } catch (err) {
      console.error("Failed to load snippets", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSnippets();
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
            {activeTab === "preferences" && <PrefsPanel />}
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
