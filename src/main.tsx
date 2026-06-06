import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Settings } from "./routes/settings";
import { FormRoute } from "./routes/form";
import "./App.css";

import { initFromConfig } from "./lib/logger";

async function main() {
  await initFromConfig();
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Navigate to="/settings" replace />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/form/:snippetId" element={<FormRoute />} />
        </Routes>
      </BrowserRouter>
    </React.StrictMode>,
  );
}
main();
