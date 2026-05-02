import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { ArrowRight, Clock, Zap, FileText, Trash2 } from "lucide-react";
import { tools, CATEGORY_ORDER, CATEGORY_LABELS } from "@/data/toolsRegistry";
import { useRecentFiles } from "@/hooks/useRecentFiles";

const QUICK_TOOLS = ["merge", "compress", "split", "protect", "watermark", "rotate"];

export default function Dashboard() {
  const navigate = useNavigate();
  const [recentFiles, clearRecent] = useRecentFiles();
  const quickTools = QUICK_TOOLS.map((id) => tools.find((t) => t.id === id)!).filter(Boolean);

  return (
    <div style={{ maxWidth: 1200, margin: "0 auto" }}>
      {/* Hero */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        style={{
          padding: "var(--sp-10) var(--sp-8)",
          borderRadius: "var(--radius-xl)",
          background: "linear-gradient(135deg, rgba(16,185,129,0.08) 0%, rgba(6,182,212,0.08) 50%, rgba(16,185,129,0.03) 100%)",
          border: "1px solid var(--border)",
          marginBottom: "var(--sp-8)",
          position: "relative",
          overflow: "hidden",
        }}
      >
        {/* Background glow */}
        <div style={{
          position: "absolute",
          top: "-50%", right: "-20%",
          width: 400, height: 400,
          borderRadius: "50%",
          background: "radial-gradient(circle, rgba(16,185,129,0.12) 0%, transparent 70%)",
          pointerEvents: "none",
        }} />

        <div style={{ position: "relative", zIndex: 1 }}>
          <h1 style={{
            fontSize: "var(--text-4xl)",
            fontWeight: 800,
            marginBottom: "var(--sp-3)",
            lineHeight: 1.2,
          }}>
            Welcome to <span className="gradient-text">PDF Office</span>
          </h1>
          <p style={{
            fontSize: "var(--text-lg)",
            color: "var(--text-secondary)",
            maxWidth: 600,
            lineHeight: 1.6,
          }}>
            Local-first, private PDF tools and office suite.
            No cloud, no subscriptions — powered by Rust.
          </p>
        </div>
      </motion.div>

      {/* Quick Actions */}
      <section style={{ marginBottom: "var(--sp-10)" }}>
        <div style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: "var(--sp-5)",
        }}>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-2)" }}>
            <Zap size={18} style={{ color: "var(--accent)" }} />
            <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700 }}>Quick Actions</h2>
          </div>
          <button className="btn btn-ghost" onClick={() => navigate("/tools")}>
            All tools <ArrowRight size={14} />
          </button>
        </div>

        <div style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(170px, 1fr))",
          gap: "var(--sp-4)",
        }}>
          {quickTools.map((tool, i) => {
            const Icon = tool.icon;
            return (
              <motion.div
                key={tool.id}
                initial={{ opacity: 0, y: 16 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.05, duration: 0.3 }}
                className="tool-card"
                onClick={() => navigate(`/tool/${tool.id}`)}
              >
                <div className="tool-icon">
                  <Icon size={22} />
                </div>
                <span className="tool-name">{tool.name}</span>
                <span className="tool-desc">{tool.description}</span>
              </motion.div>
            );
          })}
        </div>
      </section>

      {/* Recent Files */}
      {recentFiles.length > 0 && (
        <section style={{ marginBottom: "var(--sp-10)" }}>
          <div style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            marginBottom: "var(--sp-5)",
          }}>
            <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-2)" }}>
              <Clock size={18} style={{ color: "var(--text-tertiary)" }} />
              <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700 }}>Recent Files</h2>
            </div>
            <button className="btn btn-ghost" onClick={clearRecent} style={{ gap: 6, color: "var(--text-tertiary)" }}>
              <Trash2 size={14} /> Clear
            </button>
          </div>
          <div className="card" style={{ padding: 0, overflow: "hidden" }}>
            {recentFiles.slice(0, 8).map((f, i) => (
              <div key={i} className="file-item" style={{
                borderBottom: i < recentFiles.length - 1 ? "1px solid var(--border)" : "none",
                padding: "var(--sp-3) var(--sp-5)",
              }}>
                <div className="file-icon">
                  <FileText size={18} />
                </div>
                <div className="file-info">
                  <div className="file-name">{f.name}</div>
                  <div className="file-meta">
                    {f.tool.replace("/api/pdf/", "")} • {new Date(f.timestamp).toLocaleString()}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Tool Categories Grid */}
      <section>
        <h2 style={{
          fontSize: "var(--text-xl)",
          fontWeight: 700,
          marginBottom: "var(--sp-5)",
        }}>
          All Tool Categories
        </h2>

        {CATEGORY_ORDER.map((cat) => {
          const catTools = tools.filter((t) => t.category === cat);
          return (
            <div key={cat} style={{ marginBottom: "var(--sp-8)" }}>
              <h3 style={{
                fontSize: "var(--text-sm)",
                fontWeight: 600,
                color: "var(--text-tertiary)",
                textTransform: "uppercase",
                letterSpacing: "0.05em",
                marginBottom: "var(--sp-3)",
              }}>
                {CATEGORY_LABELS[cat]}
              </h3>
              <div style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fill, minmax(170px, 1fr))",
                gap: "var(--sp-3)",
              }}>
                {catTools.map((tool) => {
                  const Icon = tool.icon;
                  return (
                    <div
                      key={tool.id}
                      className="tool-card"
                      onClick={() => navigate(`/tool/${tool.id}`)}
                      style={{ padding: "var(--sp-4)" }}
                    >
                      <div className="tool-icon" style={{ width: 40, height: 40 }}>
                        <Icon size={18} />
                      </div>
                      <span className="tool-name" style={{ fontSize: "var(--text-xs)" }}>
                        {tool.name}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </section>
    </div>
  );
}
