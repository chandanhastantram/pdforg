import { useState, type ReactNode } from "react";
import Sidebar from "./Sidebar";
import Header from "./Header";

export default function AppShell({ children }: { children: ReactNode }) {
  const [collapsed, setCollapsed] = useState(false);

  return (
    <div style={{
      display: "flex",
      height: "100vh",
      overflow: "hidden",
      background: "var(--bg-primary)",
    }}>
      <Sidebar collapsed={collapsed} onToggle={() => setCollapsed(!collapsed)} />

      <div style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
        transition: "margin-left var(--duration-normal) var(--ease-out)",
      }}>
        <Header collapsed={collapsed} />

        <main style={{
          flex: 1,
          overflow: "auto",
          padding: "var(--sp-6) var(--sp-8)",
        }}>
          {children}
        </main>
      </div>
    </div>
  );
}
