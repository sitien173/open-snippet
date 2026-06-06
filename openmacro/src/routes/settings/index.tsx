import { useState, useEffect } from "react";
import { Snippet, listSnippets } from "../../lib/snippets";
import { SnippetList } from "./SnippetList";
import { SnippetEditor } from "./SnippetEditor";
import { PrefsPanel } from "./PrefsPanel";
import { SyncPanel } from "./SyncPanel";

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
    <div
      className="settings-layout"
      style={{
        display: "flex",
        minHeight: "100vh",
        backgroundColor: "#1e1e2e",
        color: "#cdd6f4",
        fontFamily: "Outfit, Inter, sans-serif",
      }}
    >
      {/* Sidebar Navigation */}
      <aside
        className="settings-sidebar"
        style={{
          width: "250px",
          backgroundColor: "#11111b",
          borderRight: "1px solid #313244",
          display: "flex",
          flexDirection: "column",
          padding: "1.5rem",
        }}
      >
        <div className="sidebar-brand" style={{ marginBottom: "2rem" }}>
          <h1 style={{ fontSize: "1.5rem", fontWeight: "bold", margin: 0, textAlign: "left" }}>
            OpenMacro
          </h1>
          <span style={{ fontSize: "0.8rem", color: "#a6adc8" }}>Settings Console</span>
        </div>

        <nav style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
          <button
            type="button"
            onClick={() => {
              setActiveTab("snippets");
              setIsEditorOpen(false);
            }}
            style={{
              padding: "0.75rem 1rem",
              borderRadius: "8px",
              border: "none",
              backgroundColor: activeTab === "snippets" && !isEditorOpen ? "#313244" : "transparent",
              color: activeTab === "snippets" && !isEditorOpen ? "#cdd6f4" : "#a6adc8",
              textAlign: "left",
              cursor: "pointer",
              boxShadow: "none",
              transition: "all 0.2s ease",
            }}
          >
            ✂ Snippets
          </button>
          <button
            type="button"
            onClick={() => {
              setActiveTab("preferences");
              setIsEditorOpen(false);
            }}
            style={{
              padding: "0.75rem 1rem",
              borderRadius: "8px",
              border: "none",
              backgroundColor: activeTab === "preferences" ? "#313244" : "transparent",
              color: activeTab === "preferences" ? "#cdd6f4" : "#a6adc8",
              textAlign: "left",
              cursor: "pointer",
              boxShadow: "none",
              transition: "all 0.2s ease",
            }}
          >
            ⚙ Preferences
          </button>
          <button
            type="button"
            onClick={() => {
              setActiveTab("sync");
              setIsEditorOpen(false);
            }}
            style={{
              padding: "0.75rem 1rem",
              borderRadius: "8px",
              border: "none",
              backgroundColor: activeTab === "sync" ? "#313244" : "transparent",
              color: activeTab === "sync" ? "#cdd6f4" : "#a6adc8",
              textAlign: "left",
              cursor: "pointer",
              boxShadow: "none",
              transition: "all 0.2s ease",
            }}
          >
            🔄 Sync & Diagnostics
          </button>
        </nav>
      </aside>

      {/* Main Content Area */}
      <main
        className="settings-content"
        style={{
          flex: 1,
          padding: "2.5rem",
          overflowY: "auto",
        }}
      >
        {isEditorOpen ? (
          <SnippetEditor
            snippet={editingSnippet}
            allSnippets={snippets}
            onSave={handleSaveEditor}
            onCancel={handleCancelEditor}
          />
        ) : (
          <>
            {loading && <div style={{ color: "#a6adc8", marginBottom: "1rem" }}>Loading snippets...</div>}
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
