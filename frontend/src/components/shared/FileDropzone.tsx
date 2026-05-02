import { useCallback } from "react";
import { useDropzone } from "react-dropzone";
import { Upload, FileText, X } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

interface Props {
  files: File[];
  onFilesChange: (files: File[]) => void;
  multiple?: boolean;
  accept?: Record<string, string[]>;
}

export default function FileDropzone({ files, onFilesChange, multiple, accept }: Props) {
  const onDrop = useCallback(
    (accepted: File[]) => {
      if (multiple) {
        onFilesChange([...files, ...accepted]);
      } else {
        onFilesChange(accepted.slice(0, 1));
      }
    },
    [files, onFilesChange, multiple]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: accept || { "application/pdf": [".pdf"] },
    multiple: multiple || false,
  });

  const removeFile = (idx: number) => {
    onFilesChange(files.filter((_, i) => i !== idx));
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div>
      <div
        {...getRootProps()}
        className={`dropzone ${isDragActive ? "active" : ""}`}
      >
        <input {...getInputProps()} />
        <div className="dropzone-icon">
          <Upload size={28} />
        </div>
        <div>
          <p style={{
            fontWeight: 600,
            fontSize: "var(--text-base)",
            color: "var(--text-primary)",
            marginBottom: "var(--sp-1)",
          }}>
            {isDragActive ? "Drop files here..." : "Drag & drop files here"}
          </p>
          <p style={{
            fontSize: "var(--text-sm)",
            color: "var(--text-tertiary)",
          }}>
            or click to browse • {multiple ? "Multiple files allowed" : "Single PDF file"}
          </p>
        </div>
      </div>

      {/* File list */}
      <AnimatePresence>
        {files.length > 0 && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            style={{
              marginTop: "var(--sp-4)",
              display: "flex",
              flexDirection: "column",
              gap: "var(--sp-2)",
            }}
          >
            {files.map((file, idx) => (
              <motion.div
                key={`${file.name}-${idx}`}
                initial={{ opacity: 0, x: -16 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -16 }}
                className="file-item"
                style={{
                  background: "var(--bg-tertiary)",
                  borderRadius: "var(--radius-md)",
                }}
              >
                <div className="file-icon">
                  <FileText size={18} />
                </div>
                <div className="file-info">
                  <div className="file-name">{file.name}</div>
                  <div className="file-meta">{formatSize(file.size)}</div>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    removeFile(idx);
                  }}
                  className="btn btn-ghost btn-icon"
                  style={{ flexShrink: 0 }}
                >
                  <X size={14} />
                </button>
              </motion.div>
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
