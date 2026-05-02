import { useState, useEffect } from "react";

export interface RecentFile {
  name: string;
  tool: string;
  timestamp: number;
}

const STORAGE_KEY = "pdf-office-recent";
const MAX_RECENT = 20;

function loadRecent(): RecentFile[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

export function addRecentFile(name: string, tool: string) {
  const list = loadRecent();
  list.unshift({ name, tool, timestamp: Date.now() });
  if (list.length > MAX_RECENT) list.length = MAX_RECENT;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
  window.dispatchEvent(new Event("recent-files-updated"));
}

export function clearRecentFiles() {
  localStorage.removeItem(STORAGE_KEY);
  window.dispatchEvent(new Event("recent-files-updated"));
}

export function useRecentFiles(): [RecentFile[], () => void] {
  const [files, setFiles] = useState<RecentFile[]>(loadRecent);

  useEffect(() => {
    const handler = () => setFiles(loadRecent());
    window.addEventListener("recent-files-updated", handler);
    return () => window.removeEventListener("recent-files-updated", handler);
  }, []);

  return [files, clearRecentFiles];
}
