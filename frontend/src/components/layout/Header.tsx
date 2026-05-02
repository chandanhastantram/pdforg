import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Search, X } from "lucide-react";
import { tools } from "@/data/toolsRegistry";

interface Props {
  collapsed: boolean;
}

export default function Header({ collapsed: _collapsed }: Props) {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [focused, setFocused] = useState(false);

  const results = query.length > 0
    ? tools.filter(
        (t) =>
          t.name.toLowerCase().includes(query.toLowerCase()) ||
          t.description.toLowerCase().includes(query.toLowerCase())
      ).slice(0, 6)
    : [];

  return (
    <header style={{
      height: "var(--header-height)",
      display: "flex",
      alignItems: "center",
      padding: "0 var(--sp-8)",
      borderBottom: "1px solid var(--border)",
      background: "var(--bg-secondary)",
      gap: "var(--sp-4)",
      flexShrink: 0,
      position: "relative",
      zIndex: 20,
    }}>
      {/* Search */}
      <div style={{ position: "relative", flex: 1, maxWidth: 480 }}>
        <div style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--sp-2)",
          padding: "var(--sp-2) var(--sp-3)",
          background: "var(--bg-tertiary)",
          border: `1px solid ${focused ? "var(--accent)" : "var(--border)"}`,
          borderRadius: "var(--radius-md)",
          transition: "border-color var(--duration-fast)",
        }}>
          <Search size={16} style={{ color: "var(--text-tertiary)", flexShrink: 0 }} />
          <input
            type="text"
            placeholder="Search tools... (e.g. merge, compress, watermark)"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onFocus={() => setFocused(true)}
            onBlur={() => setTimeout(() => setFocused(false), 200)}
            style={{
              flex: 1,
              background: "none",
              border: "none",
              outline: "none",
              color: "var(--text-primary)",
              fontFamily: "var(--font-sans)",
              fontSize: "var(--text-sm)",
            }}
          />
          {query && (
            <button
              onClick={() => setQuery("")}
              style={{
                background: "none", border: "none",
                color: "var(--text-tertiary)", cursor: "pointer",
                display: "flex", padding: 2,
              }}
            >
              <X size={14} />
            </button>
          )}
        </div>

        {/* Search results dropdown */}
        {focused && results.length > 0 && (
          <div style={{
            position: "absolute",
            top: "calc(100% + 4px)",
            left: 0,
            right: 0,
            background: "var(--bg-elevated)",
            border: "1px solid var(--border)",
            borderRadius: "var(--radius-md)",
            boxShadow: "var(--shadow-lg)",
            overflow: "hidden",
            zIndex: 100,
            animation: "fadeInDown 0.15s var(--ease-out)",
          }}>
            {results.map((tool) => {
              const Icon = tool.icon;
              return (
                <button
                  key={tool.id}
                  onClick={() => {
                    navigate(`/tool/${tool.id}`);
                    setQuery("");
                  }}
                  style={{
                    width: "100%",
                    display: "flex",
                    alignItems: "center",
                    gap: "var(--sp-3)",
                    padding: "var(--sp-3) var(--sp-4)",
                    background: "none",
                    border: "none",
                    color: "var(--text-primary)",
                    cursor: "pointer",
                    fontFamily: "var(--font-sans)",
                    fontSize: "var(--text-sm)",
                    textAlign: "left",
                    transition: "background var(--duration-fast)",
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.background = "var(--surface-hover)")}
                  onMouseLeave={(e) => (e.currentTarget.style.background = "none")}
                >
                  <Icon size={16} style={{ color: "var(--accent)", flexShrink: 0 }} />
                  <div>
                    <div style={{ fontWeight: 500 }}>{tool.name}</div>
                    <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
                      {tool.description}
                    </div>
                  </div>
                </button>
              );
            })}
          </div>
        )}
      </div>

      {/* Right side info */}
      <div style={{
        display: "flex",
        alignItems: "center",
        gap: "var(--sp-3)",
        color: "var(--text-tertiary)",
        fontSize: "var(--text-xs)",
        fontFamily: "var(--font-mono)",
      }}>
        <span className="badge">v0.2.1</span>
        <span style={{ opacity: 0.5 }}>Rust-powered</span>
      </div>
    </header>
  );
}
