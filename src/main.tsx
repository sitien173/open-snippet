import React, { useEffect } from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route, Navigate, useNavigate } from "react-router-dom";
import { Settings } from "./routes/settings";
import { FormRoute } from "./routes/form";
import { LogsRoute } from "./routes/logs";
import "./App.css";

import { initFromConfig } from "./lib/logger";

function HotkeyHandler({ children }: { children: React.ReactNode }) {
  const navigate = useNavigate();

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
      const modifier = isMac ? e.metaKey : e.ctrlKey;
      if (modifier && e.shiftKey && (e.key === "l" || e.key === "L")) {
        const target = e.target as HTMLElement | null;
        if (
          target &&
          (target.tagName === "INPUT" ||
            target.tagName === "TEXTAREA" ||
            target.isContentEditable)
        ) {
          return;
        }
        e.preventDefault();
        navigate("/logs");
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [navigate]);

  return <>{children}</>;
}

async function main() {
  await initFromConfig();
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <BrowserRouter>
        <HotkeyHandler>
          <Routes>
            <Route path="/" element={<Navigate to="/settings" replace />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/form/:snippetId" element={<FormRoute />} />
            <Route path="/logs" element={<LogsRoute />} />
          </Routes>
        </HotkeyHandler>
      </BrowserRouter>
    </React.StrictMode>,
  );
}
main();

