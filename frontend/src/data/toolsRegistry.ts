import {
  Merge, Scissors, Minimize2, RotateCw, FileOutput, ImagePlus,
  FileText, Lock, Unlock, Droplets, Hash, FileSearch, Shield,
  Layers, BookOpen, ArrowDownUp, Stamp, Type, Eraser, ScanSearch
} from "lucide-react";
import type { ComponentType } from "react";

export interface ToolDef {
  id: string;
  name: string;
  description: string;
  category: ToolCategory;
  icon: ComponentType<{ size?: number }>;
  endpoint: string;
  /** Fields the tool accepts (beside the file upload) */
  fields?: ToolField[];
  /** Does this tool accept multiple files? */
  multiFile?: boolean;
  /** Accepted MIME types */
  accept?: Record<string, string[]>;
}

export interface ToolField {
  name: string;
  label: string;
  type: "text" | "number" | "select" | "checkbox";
  default?: string;
  options?: { value: string; label: string }[];
  placeholder?: string;
}

export type ToolCategory =
  | "organize"
  | "convert"
  | "security"
  | "edit"
  | "analyze"
  | "other";

export const CATEGORY_LABELS: Record<ToolCategory, string> = {
  organize: "Organize",
  convert: "Convert",
  security: "Security",
  edit: "Edit & Stamp",
  analyze: "Analyze",
  other: "Other",
};

export const CATEGORY_ORDER: ToolCategory[] = [
  "organize", "convert", "security", "edit", "analyze", "other"
];

export const tools: ToolDef[] = [
  // ─── Organize ──────────────────────────────────
  {
    id: "merge",
    name: "Merge PDFs",
    description: "Combine multiple PDFs into a single file",
    category: "organize",
    icon: Merge,
    endpoint: "/api/pdf/merge",
    multiFile: true,
  },
  {
    id: "split",
    name: "Split PDF",
    description: "Split a PDF into multiple parts",
    category: "organize",
    icon: Scissors,
    endpoint: "/api/pdf/split",
    fields: [
      {
        name: "method", label: "Split method", type: "select", default: "bypage",
        options: [
          { value: "bypage", label: "By page number" },
          { value: "range", label: "By range" },
        ],
      },
      { name: "value", label: "Page / Range", type: "text", default: "1", placeholder: "e.g. 3 or 1-5" },
    ],
  },
  {
    id: "rotate",
    name: "Rotate PDF",
    description: "Rotate pages by 90°, 180° or 270°",
    category: "organize",
    icon: RotateCw,
    endpoint: "/api/pdf/rotate",
    fields: [
      {
        name: "degrees", label: "Degrees", type: "select", default: "90",
        options: [
          { value: "90", label: "90° Clockwise" },
          { value: "180", label: "180°" },
          { value: "270", label: "270° Counter-clockwise" },
        ],
      },
      { name: "pages", label: "Pages", type: "text", default: "all", placeholder: "all or 1,3,5" },
    ],
  },
  {
    id: "extract",
    name: "Extract Pages",
    description: "Extract specific pages from a PDF",
    category: "organize",
    icon: FileOutput,
    endpoint: "/api/pdf/extract-pages",
    fields: [
      { name: "pages", label: "Pages to extract", type: "text", default: "1", placeholder: "e.g. 1,3,5 or 2-8" },
    ],
  },
  {
    id: "delete-pages",
    name: "Delete Pages",
    description: "Remove specific pages from a PDF",
    category: "organize",
    icon: Eraser,
    endpoint: "/api/pdf/delete-pages",
    fields: [
      { name: "pages", label: "Pages to delete", type: "text", default: "1", placeholder: "e.g. 1,3,5" },
    ],
  },
  {
    id: "insert-page",
    name: "Insert Blank Page",
    description: "Insert a blank page after a specific page",
    category: "organize",
    icon: BookOpen,
    endpoint: "/api/pdf/insert-page",
    fields: [
      { name: "after", label: "Insert after page", type: "number", default: "0" },
    ],
  },

  // ─── Convert ───────────────────────────────────
  {
    id: "images-to-pdf",
    name: "Images to PDF",
    description: "Convert images (JPEG/PNG) into a PDF",
    category: "convert",
    icon: ImagePlus,
    endpoint: "/api/pdf/images-to-pdf",
    multiFile: true,
    accept: { "image/*": [".jpg", ".jpeg", ".png"] },
  },
  {
    id: "convert-file",
    name: "Convert Document",
    description: "Convert DOCX/ODT/RTF to PDF or DOCX",
    category: "convert",
    icon: ArrowDownUp,
    endpoint: "/api/convert",
    fields: [
      {
        name: "format", label: "Output format", type: "select", default: "pdf",
        options: [
          { value: "pdf", label: "PDF" },
          { value: "docx", label: "DOCX" },
        ],
      },
    ],
    accept: {
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document": [".docx"],
      "application/rtf": [".rtf"],
      "application/vnd.oasis.opendocument.text": [".odt"],
    },
  },

  // ─── Security ──────────────────────────────────
  {
    id: "protect",
    name: "Protect PDF",
    description: "Add password and restrict permissions",
    category: "security",
    icon: Lock,
    endpoint: "/api/pdf/protect",
    fields: [
      { name: "open_password", label: "Open password", type: "text", placeholder: "Leave blank for no open password" },
      { name: "owner_password", label: "Owner password", type: "text", default: "owner" },
      { name: "no_print", label: "Disable printing", type: "checkbox" },
      { name: "no_copy", label: "Disable copying", type: "checkbox" },
      { name: "no_edit", label: "Disable editing", type: "checkbox" },
    ],
  },
  {
    id: "unlock",
    name: "Unlock PDF",
    description: "Remove password protection from a PDF",
    category: "security",
    icon: Unlock,
    endpoint: "/api/pdf/unlock",
    fields: [
      { name: "password", label: "PDF password", type: "text", placeholder: "Enter the current password" },
    ],
  },
  {
    id: "sanitize",
    name: "Sanitize PDF",
    description: "Remove hidden metadata, JS, and attachments",
    category: "security",
    icon: Shield,
    endpoint: "/api/pdf/sanitize",
    fields: [
      { name: "metadata", label: "Strip metadata", type: "checkbox", default: "1" },
      { name: "javascript", label: "Strip JavaScript", type: "checkbox", default: "1" },
      { name: "attachments", label: "Strip attachments", type: "checkbox", default: "1" },
    ],
  },

  // ─── Edit & Stamp ─────────────────────────────
  {
    id: "compress",
    name: "Compress PDF",
    description: "Reduce file size without losing quality",
    category: "edit",
    icon: Minimize2,
    endpoint: "/api/pdf/compress",
    fields: [
      {
        name: "level", label: "Compression level", type: "select", default: "medium",
        options: [
          { value: "low", label: "Low (faster)" },
          { value: "medium", label: "Medium (balanced)" },
          { value: "high", label: "High (smaller file)" },
        ],
      },
    ],
  },
  {
    id: "watermark",
    name: "Add Watermark",
    description: "Overlay text watermark on every page",
    category: "edit",
    icon: Droplets,
    endpoint: "/api/pdf/watermark",
    fields: [
      { name: "text", label: "Watermark text", type: "text", default: "CONFIDENTIAL" },
      { name: "opacity", label: "Opacity (0-1)", type: "text", default: "0.3" },
      { name: "font_size", label: "Font size", type: "number", default: "72" },
      {
        name: "position", label: "Position", type: "select", default: "center",
        options: [
          { value: "center", label: "Center" },
          { value: "top-left", label: "Top Left" },
          { value: "bottom-right", label: "Bottom Right" },
        ],
      },
    ],
  },
  {
    id: "page-numbers",
    name: "Add Page Numbers",
    description: "Stamp page numbers on every page",
    category: "edit",
    icon: Hash,
    endpoint: "/api/pdf/page-numbers",
    fields: [
      {
        name: "position", label: "Position", type: "select", default: "bottom-center",
        options: [
          { value: "bottom-center", label: "Bottom Center" },
          { value: "bottom-right", label: "Bottom Right" },
          { value: "top-right", label: "Top Right" },
        ],
      },
      { name: "start", label: "Start number", type: "number", default: "1" },
      { name: "font_size", label: "Font size", type: "number", default: "10" },
    ],
  },
  {
    id: "header-footer",
    name: "Header & Footer",
    description: "Add custom headers and footers",
    category: "edit",
    icon: Type,
    endpoint: "/api/pdf/header-footer",
    fields: [
      { name: "header_center", label: "Header text", type: "text", placeholder: "Header text" },
      { name: "footer_center", label: "Footer text", type: "text", placeholder: "Footer text" },
      { name: "font_size", label: "Font size", type: "number", default: "10" },
    ],
  },
  {
    id: "bates",
    name: "Bates Numbering",
    description: "Add Bates numbers for legal documents",
    category: "edit",
    icon: Stamp,
    endpoint: "/api/pdf/bates",
    fields: [
      { name: "prefix", label: "Prefix", type: "text", default: "DOC-" },
      { name: "start", label: "Start number", type: "number", default: "1" },
      { name: "digits", label: "Digits", type: "number", default: "6" },
    ],
  },
  {
    id: "flatten",
    name: "Flatten PDF",
    description: "Flatten all form fields and annotations",
    category: "edit",
    icon: Layers,
    endpoint: "/api/pdf/flatten",
  },

  // ─── Analyze ──────────────────────────────────
  {
    id: "metadata",
    name: "View Metadata",
    description: "Inspect PDF metadata (title, author, dates)",
    category: "analyze",
    icon: FileSearch,
    endpoint: "/api/pdf/metadata",
  },
  {
    id: "compare",
    name: "Compare PDFs",
    description: "Compare two PDFs page by page",
    category: "analyze",
    icon: ScanSearch,
    endpoint: "/api/pdf/compare",
    multiFile: true,
  },
];

export function getToolById(id: string): ToolDef | undefined {
  return tools.find((t) => t.id === id);
}

export function getToolsByCategory(cat: ToolCategory): ToolDef[] {
  return tools.filter((t) => t.category === cat);
}
