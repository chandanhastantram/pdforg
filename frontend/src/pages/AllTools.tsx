import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Search } from "lucide-react";
import { tools, CATEGORY_ORDER, CATEGORY_LABELS } from "@/data/toolsRegistry";

export default function AllTools() {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");

  const filtered = query
    ? tools.filter(
        (t) =>
          t.name.toLowerCase().includes(query.toLowerCase()) ||
          t.description.toLowerCase().includes(query.toLowerCase())
      )
    : tools;

  const categories = query
    ? [{ cat: "all" as const, label: "Search Results", items: filtered }]
    : CATEGORY_ORDER.map((cat) => ({
        cat,
        label: CATEGORY_LABELS[cat],
        items: filtered.filter((t) => t.category === cat),
      })).filter((c) => c.items.length > 0);

  return (
    <div style={{ maxWidth: 1200, margin: "0 auto" }}>
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
      >
        <h1 style={{
          fontSize: "var(--text-3xl)",
          fontWeight: 800,
          marginBottom: "var(--sp-2)",
        }}>
          All Tools
        </h1>
        <p style={{
          color: "var(--text-secondary)",
          marginBottom: "var(--sp-6)",
        }}>
          {tools.length} powerful PDF tools — all running locally on your machine
        </p>
      </motion.div>

      {/* Search */}
      <div style={{
        display: "flex",
        alignItems: "center",
        gap: "var(--sp-2)",
        padding: "var(--sp-3) var(--sp-4)",
        background: "var(--bg-secondary)",
        border: "1px solid var(--border)",
        borderRadius: "var(--radius-lg)",
        marginBottom: "var(--sp-8)",
        maxWidth: 480,
      }}>
        <Search size={18} style={{ color: "var(--text-tertiary)" }} />
        <input
          type="text"
          placeholder="Filter tools..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          style={{
            flex: 1,
            background: "none",
            border: "none",
            outline: "none",
            color: "var(--text-primary)",
            fontFamily: "var(--font-sans)",
            fontSize: "var(--text-base)",
          }}
        />
      </div>

      {/* Categories */}
      {categories.map(({ cat, label, items }) => (
        <section key={cat} style={{ marginBottom: "var(--sp-10)" }}>
          <h2 style={{
            fontSize: "var(--text-sm)",
            fontWeight: 600,
            color: "var(--text-tertiary)",
            textTransform: "uppercase",
            letterSpacing: "0.05em",
            marginBottom: "var(--sp-4)",
            paddingBottom: "var(--sp-2)",
            borderBottom: "1px solid var(--border)",
          }}>
            {label}
            <span style={{
              marginLeft: "var(--sp-2)",
              color: "var(--text-tertiary)",
              fontWeight: 400,
            }}>
              ({items.length})
            </span>
          </h2>

          <div style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))",
            gap: "var(--sp-4)",
          }}>
            {items.map((tool, i) => {
              const Icon = tool.icon;
              return (
                <motion.div
                  key={tool.id}
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: i * 0.03, duration: 0.25 }}
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
      ))}

      {filtered.length === 0 && (
        <div style={{
          textAlign: "center",
          padding: "var(--sp-16)",
          color: "var(--text-tertiary)",
        }}>
          <p style={{ fontSize: "var(--text-lg)" }}>No tools match "{query}"</p>
        </div>
      )}
    </div>
  );
}
