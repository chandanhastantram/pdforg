import axios from "axios";

const api = axios.create({
  baseURL: "",
  timeout: 300_000, // 5 minutes for very large PDFs
});

api.interceptors.response.use(
  (r) => r,
  async (err) => {
    // Try to get the real error message from the response body
    let msg = "An unexpected error occurred";
    if (err.response) {
      // Server responded with error status
      const data = err.response.data;
      if (typeof data === "string" && data.length > 0) {
        msg = data;
      } else if (data instanceof Blob) {
        try {
          const text = await data.text();
          if (text.length > 0) msg = text;
        } catch {}
      } else if (data?.message) {
        msg = data.message;
      } else {
        msg = `Server error ${err.response.status}`;
      }
    } else if (err.request) {
      // Request was made but no response received
      msg = "Cannot reach the PDF Office backend. Make sure it is running on port 3847.";
    } else {
      msg = err.message || msg;
    }
    return Promise.reject(new Error(msg));
  }
);

/** Upload file(s) to a PDF tool endpoint and return the result as a Blob */
export async function processPdf(
  endpoint: string,
  files: File[],
  fields?: Record<string, string>,
  onProgress?: (pct: number) => void
): Promise<Blob> {
  const form = new FormData();
  // Append every file under the "file" field name — backend iterates all "file" fields
  files.forEach((f) => form.append("file", f));
  if (fields) {
    Object.entries(fields).forEach(([k, v]) => form.append(k, v));
  }
  const res = await api.post(endpoint, form, {
    responseType: "blob",
    onUploadProgress: (e) => {
      if (e.total && onProgress) onProgress(Math.round((e.loaded * 100) / e.total));
    },
  });
  return res.data;
}

/** Upload file and get JSON response (e.g. metadata, compare) */
export async function processPdfJson<T = unknown>(
  endpoint: string,
  files: File[],
  fields?: Record<string, string>
): Promise<T> {
  const form = new FormData();
  files.forEach((f) => form.append("file", f));
  if (fields) {
    Object.entries(fields).forEach(([k, v]) => form.append(k, v));
  }
  const res = await api.post(endpoint, form);
  return res.data;
}

/** Trigger a file download from a Blob */
export function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

export default api;
