import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import UtilityDashboard from "./pages/UtilityDashboard";
import RevenueDashboard from "./pages/RevenueDashboard";
import FootballDashboard from "./pages/FootballDashboard";
import "./App.css";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Navigate to="/utility" replace />} />
        <Route path="/utility/*" element={<UtilityDashboard />} />
        <Route path="/revenue/*" element={<RevenueDashboard />} />
        <Route path="/football" element={<FootballDashboard />} />
      </Routes>
    </BrowserRouter>
  );
}
