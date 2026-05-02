import { useParams, useNavigate } from "react-router-dom";
import { useState } from "react";
import { motion } from "framer-motion";
import { ArrowLeft, Play, Loader } from "lucide-react";
import { getToolById } from "@/data/toolsRegistry";
import FileDropzone from "@/components/shared/FileDropzone";
import { useToolExecution } from "@/hooks/useToolExecution";

export default function ToolPage() {
  const { toolId } = useParams<{ toolId: string }>();
  const navigate = useNavigate();
  const tool = getToolById(toolId || "");

  const [files, setFiles] = useState<File[]>([]);
  const [fieldValues, setFieldValues] = useState<Record<string, string>>({});

  // Initialize defaults
  const getFieldValue = (name: string, defaultVal?: string) =>
    fieldValues[name] ?? defaultVal ?? "";

  const setFieldValue = (name: string, value: string) =>
    setFieldValues((prev) => ({ ...prev, [name]: value }));

  const isJsonTool = tool?.id === "metadata" || tool?.id === "compare";

  const { execute, loading, progress, result } = useToolExecution({
    endpoint: tool?.endpoint || "",
    outputFilename: undefined,
    jsonResponse: isJsonTool,
  });

  if (!tool) {
    return (
      <div style={{ textAlign: "center", padding: "var(--sp-16)" }}>
        <h2 style={{ fontSize: "var(--text-2xl)", marginBottom: "var(--sp-4)" }}>
          Tool not found
        </h2>
        <button className="btn btn-primary" onClick={() => navigate("/")}>
          Go Home
        </button>
      </div>
    );
  }

  const Icon = tool.icon;

  const handleProcess = () => {
    if (files.length === 0) return;
    const fields: Record<string, string> = {};
    tool.fields?.forEach((f) => {
      const val = getFieldValue(f.name, f.default);
      if (f.type === "checkbox") {
        fields[f.name] = val === "1" || val === "true" ? "1" : "0";
      } else if (val) {
        fields[f.name] = val;
      }
    });
    execute(files, Object.keys(fields).length > 0 ? fields : undefined);
  };

  return (
    <div style={{ maxWidth: 720, margin: "0 auto" }}>
      {/* Back button */}
      <button
        className="btn btn-ghost"
        onClick={() => navigate(-1)}
        style={{ marginBottom: "var(--sp-4)", gap: "var(--sp-2)" }}
      >
        <ArrowLeft size={16} /> Back
      </button>

      {/* Tool header */}
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--sp-4)",
          marginBottom: "var(--sp-6)",
        }}
      >
        <div style={{
          width: 56, height: 56,
          borderRadius: "var(--radius-lg)",
          background: "var(--accent-muted)",
          color: "var(--accent)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}>
          <Icon size={28} />
        </div>
        <div>
          <h1 style={{ fontSize: "var(--text-2xl)", fontWeight: 700 }}>{tool.name}</h1>
          <p style={{ color: "var(--text-secondary)", fontSize: "var(--text-sm)" }}>
            {tool.description}
          </p>
        </div>
      </motion.div>

      {/* File Upload */}
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        style={{ marginBottom: "var(--sp-6)" }}
      >
        <FileDropzone
          files={files}
          onFilesChange={setFiles}
          multiple={tool.multiFile}
          accept={tool.accept}
        />
      </motion.div>

      {/* Tool Options */}
      {tool.fields && tool.fields.length > 0 && (
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="card"
          style={{ marginBottom: "var(--sp-6)" }}
        >
          <h3 style={{
            fontSize: "var(--text-sm)",
            fontWeight: 600,
            color: "var(--text-tertiary)",
            textTransform: "uppercase",
            letterSpacing: "0.05em",
            marginBottom: "var(--sp-4)",
          }}>
            Options
          </h3>

          <div style={{
            display: "flex",
            flexDirection: "column",
            gap: "var(--sp-4)",
          }}>
            {tool.fields.map((field) => (
              <div key={field.name}>
                <label style={{
                  display: "block",
                  fontSize: "var(--text-sm)",
                  fontWeight: 500,
                  color: "var(--text-secondary)",
                  marginBottom: "var(--sp-1)",
                }}>
                  {field.label}
                </label>

                {field.type === "select" ? (
                  <select
                    className="select"
                    value={getFieldValue(field.name, field.default)}
                    onChange={(e) => setFieldValue(field.name, e.target.value)}
                  >
                    {field.options?.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                ) : field.type === "checkbox" ? (
                  <label style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "var(--sp-2)",
                    cursor: "pointer",
                  }}>
                    <input
                      type="checkbox"
                      checked={getFieldValue(field.name, field.default) === "1"}
                      onChange={(e) => setFieldValue(field.name, e.target.checked ? "1" : "0")}
                      style={{
                        width: 18, height: 18,
                        accentColor: "var(--accent)",
                        cursor: "pointer",
                      }}
                    />
                    <span style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>
                      {field.label}
                    </span>
                  </label>
                ) : (
                  <input
                    type={field.type}
                    className="input"
                    value={getFieldValue(field.name, field.default)}
                    onChange={(e) => setFieldValue(field.name, e.target.value)}
                    placeholder={field.placeholder}
                  />
                )}
              </div>
            ))}
          </div>
        </motion.div>
      )}

      {/* Process button */}
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
      >
        <button
          className="btn btn-primary btn-lg"
          onClick={handleProcess}
          disabled={files.length === 0 || loading}
          style={{ width: "100%", fontSize: "var(--text-base)" }}
        >
          {loading ? (
            <>
              <Loader size={18} className="animate-spin" />
              Processing{progress > 0 ? ` (${progress}%)` : "..."}
            </>
          ) : (
            <>
              <Play size={18} />
              Process {tool.name}
            </>
          )}
        </button>

        {/* Progress bar */}
        {loading && progress > 0 && (
          <div className="progress-bar" style={{ marginTop: "var(--sp-3)" }}>
            <div className="progress-fill" style={{ width: `${progress}%` }} />
          </div>
        )}
      </motion.div>

      {/* JSON result display (for metadata, compare) */}
      {result && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="card"
          style={{ marginTop: "var(--sp-6)" }}
        >
          <h3 style={{
            fontSize: "var(--text-sm)",
            fontWeight: 600,
            marginBottom: "var(--sp-3)",
            color: "var(--text-tertiary)",
          }}>
            Result
          </h3>
          <pre style={{
            fontFamily: "var(--font-mono)",
            fontSize: "var(--text-xs)",
            color: "var(--text-secondary)",
            background: "var(--bg-tertiary)",
            padding: "var(--sp-4)",
            borderRadius: "var(--radius-md)",
            overflow: "auto",
            maxHeight: 400,
          }}>
            {JSON.stringify(result, null, 2)}
          </pre>
        </motion.div>
      )}
    </div>
  );
}
