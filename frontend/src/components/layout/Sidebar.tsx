import { useLocation, useNavigate } from "react-router-dom";
import {
  Home, Wrench, ChevronLeft, ChevronRight,
  Sun, Moon, FileText
} from "lucide-react";
import { useTheme } from "@/hooks/useTheme";
import { tools, CATEGORY_ORDER, CATEGORY_LABELS, type ToolCategory } from "@/data/toolsRegistry";
import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface Props {
  collapsed: boolean;
  onToggle: () => void;
}

export default function Sidebar({ collapsed, onToggle }: Props) {
  const navigate = useNavigate();
  const location = useLocation();
  const { theme, toggle } = useTheme();
  const [expanded, setExpanded] = useState<ToolCategory | null>(null);

  const isActive = (path: string) => location.pathname === path;
  const isToolActive = (id: string) => location.pathname === `/tool/${id}`;

  const toolsByCategory = CATEGORY_ORDER.map((cat) => ({
    cat,
    label: CATEGORY_LABELS[cat],
    items: tools.filter((t) => t.category === cat),
  }));

  return (
    <aside style={{
      width: collapsed ? "var(--sidebar-collapsed)" : "var(--sidebar-width)",
      minWidth: collapsed ? "var(--sidebar-collapsed)" : "var(--sidebar-width)",
      height: "100vh",
      background: "var(--bg-secondary)",
      borderRight: "1px solid var(--border)",
      display: "flex",
      flexDirection: "column",
      transition: "all var(--duration-normal) var(--ease-out)",
      overflow: "hidden",
      position: "relative",
      zIndex: 10,
    }}>
      {/* Logo */}
      <div
        style={{
          height: "var(--header-height)",
          display: "flex",
          alignItems: "center",
          padding: collapsed ? "0 var(--sp-4)" : "0 var(--sp-5)",
          gap: "var(--sp-3)",
          borderBottom: "1px solid var(--border)",
          cursor: "pointer",
          flexShrink: 0,
        }}
        onClick={() => navigate("/")}
      >
        <div style={{
          width: 36, height: 36,
          borderRadius: "var(--radius-md)",
          background: "linear-gradient(135deg, var(--accent) 0%, #06b6d4 100%)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexShrink: 0,
        }}>
          <FileText size={20} color="#fff" />
        </div>
        {!collapsed && (
          <motion.span
            initial={{ opacity: 0, x: -8 }}
            animate={{ opacity: 1, x: 0 }}
            style={{ fontWeight: 700, fontSize: "var(--text-lg)", whiteSpace: "nowrap" }}
          >
            PDF Office
          </motion.span>
        )}
      </div>

      {/* Navigation */}
      <nav style={{
        flex: 1,
        overflowY: "auto",
        overflowX: "hidden",
        padding: "var(--sp-3)",
      }}>
        {/* Home */}
        <NavItem
          icon={<Home size={18} />}
          label="Dashboard"
          collapsed={collapsed}
          active={isActive("/")}
          onClick={() => navigate("/")}
        />
        <NavItem
          icon={<Wrench size={18} />}
          label="All Tools"
          collapsed={collapsed}
          active={isActive("/tools")}
          onClick={() => navigate("/tools")}
        />

        <div style={{
          height: 1,
          background: "var(--border)",
          margin: "var(--sp-3) var(--sp-2)",
        }} />

        {/* Tool categories */}
        {!collapsed && toolsByCategory.map(({ cat, label, items }) => (
          <div key={cat} style={{ marginBottom: "var(--sp-1)" }}>
            <button
              onClick={() => setExpanded(expanded === cat ? null : cat)}
              style={{
                width: "100%",
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                padding: "var(--sp-2) var(--sp-3)",
                background: "none",
                border: "none",
                color: "var(--text-tertiary)",
                fontSize: "var(--text-xs)",
                fontWeight: 600,
                fontFamily: "var(--font-sans)",
                textTransform: "uppercase",
                letterSpacing: "0.05em",
                cursor: "pointer",
                borderRadius: "var(--radius-sm)",
                transition: "color var(--duration-fast)",
              }}
              onMouseEnter={(e) => (e.currentTarget.style.color = "var(--text-secondary)")}
              onMouseLeave={(e) => (e.currentTarget.style.color = "var(--text-tertiary)")}
            >
              {label}
              <ChevronRight
                size={12}
                style={{
                  transform: expanded === cat ? "rotate(90deg)" : "none",
                  transition: "transform var(--duration-fast)",
                }}
              />
            </button>

            <AnimatePresence>
              {expanded === cat && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: "auto", opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  style={{ overflow: "hidden" }}
                >
                  {items.map((tool) => {
                    const Icon = tool.icon;
                    return (
                      <NavItem
                        key={tool.id}
                        icon={<Icon size={16} />}
                        label={tool.name}
                        collapsed={false}
                        active={isToolActive(tool.id)}
                        onClick={() => navigate(`/tool/${tool.id}`)}
                        indent
                      />
                    );
                  })}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        ))}
      </nav>

      {/* Bottom controls */}
      <div style={{
        padding: "var(--sp-3)",
        borderTop: "1px solid var(--border)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--sp-1)",
        flexShrink: 0,
      }}>
        <NavItem
          icon={theme === "dark" ? <Sun size={18} /> : <Moon size={18} />}
          label={theme === "dark" ? "Light Mode" : "Dark Mode"}
          collapsed={collapsed}
          active={false}
          onClick={toggle}
        />
        <button
          onClick={onToggle}
          className="btn btn-ghost"
          style={{
            justifyContent: collapsed ? "center" : "flex-start",
            width: "100%",
            padding: "var(--sp-2) var(--sp-3)",
          }}
        >
          {collapsed ? <ChevronRight size={18} /> : <ChevronLeft size={18} />}
          {!collapsed && <span style={{ fontSize: "var(--text-sm)" }}>Collapse</span>}
        </button>
      </div>
    </aside>
  );
}

/* ─── Nav Item ──────────────────────────────────────────── */

function NavItem({
  icon, label, collapsed, active, onClick, indent,
}: {
  icon: React.ReactNode;
  label: string;
  collapsed: boolean;
  active: boolean;
  onClick: () => void;
  indent?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      title={collapsed ? label : undefined}
      style={{
        width: "100%",
        display: "flex",
        alignItems: "center",
        gap: "var(--sp-3)",
        padding: collapsed
          ? "var(--sp-2)"
          : `var(--sp-2) var(--sp-3)`,
        paddingLeft: indent ? "var(--sp-8)" : undefined,
        justifyContent: collapsed ? "center" : "flex-start",
        background: active ? "var(--accent-muted)" : "transparent",
        color: active ? "var(--accent)" : "var(--text-secondary)",
        border: "none",
        borderRadius: "var(--radius-md)",
        cursor: "pointer",
        fontFamily: "var(--font-sans)",
        fontSize: "var(--text-sm)",
        fontWeight: active ? 600 : 400,
        transition: "all var(--duration-fast) var(--ease-out)",
        whiteSpace: "nowrap",
        overflow: "hidden",
      }}
      onMouseEnter={(e) => {
        if (!active) {
          e.currentTarget.style.background = "var(--surface-hover)";
          e.currentTarget.style.color = "var(--text-primary)";
        }
      }}
      onMouseLeave={(e) => {
        if (!active) {
          e.currentTarget.style.background = "transparent";
          e.currentTarget.style.color = "var(--text-secondary)";
        }
      }}
    >
      <span style={{ flexShrink: 0, display: "flex", alignItems: "center" }}>{icon}</span>
      {!collapsed && <span className="truncate">{label}</span>}
    </button>
  );
}
