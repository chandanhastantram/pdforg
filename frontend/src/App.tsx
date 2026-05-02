import { Routes, Route } from "react-router-dom";
import { ThemeProvider } from "@/hooks/useTheme";
import { ToastProvider } from "@/hooks/useToast";
import AppShell from "@/components/layout/AppShell";
import Dashboard from "@/pages/Dashboard";
import ToolPage from "@/pages/ToolPage";
import AllTools from "@/pages/AllTools";

export default function App() {
  return (
    <ThemeProvider>
      <ToastProvider>
        <AppShell>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/tools" element={<AllTools />} />
            <Route path="/tool/:toolId" element={<ToolPage />} />
          </Routes>
        </AppShell>
      </ToastProvider>
    </ThemeProvider>
  );
}
