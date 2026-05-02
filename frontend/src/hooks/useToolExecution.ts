import { useState, useCallback } from "react";
import { processPdf, processPdfJson, downloadBlob } from "@/services/api";
import { useToast } from "@/hooks/useToast";
import { addRecentFile } from "@/hooks/useRecentFiles";

interface UseToolOptions {
  endpoint: string;
  outputFilename?: string;
  jsonResponse?: boolean;
}

export function useToolExecution({ endpoint, outputFilename, jsonResponse }: UseToolOptions) {
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [result, setResult] = useState<unknown>(null);
  const { toast } = useToast();

  const execute = useCallback(
    async (files: File[], fields?: Record<string, string>) => {
      setLoading(true);
      setProgress(0);
      setResult(null);
      try {
        if (jsonResponse) {
          const data = await processPdfJson(endpoint, files, fields);
          setResult(data);
          toast("Processing complete!", "success");
        } else {
          const blob = await processPdf(endpoint, files, fields, setProgress);
          const fname = outputFilename || `processed_${files[0]?.name || "output.pdf"}`;
          downloadBlob(blob, fname);
          addRecentFile(files[0]?.name || "unknown", endpoint);
          toast(`Downloaded ${fname}`, "success");
        }
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : "Processing failed";
        toast(msg, "error");
      } finally {
        setLoading(false);
        setProgress(0);
      }
    },
    [endpoint, outputFilename, jsonResponse, toast]
  );

  return { execute, loading, progress, result };
}
