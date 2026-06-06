import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Settings } from "./routes/settings";
import { FormRoute } from "./routes/form";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/settings" element={<Settings />} />
        <Route path="/form/:snippetId" element={<FormRoute />} />
      </Routes>
    </BrowserRouter>
  </React.StrictMode>,
);
