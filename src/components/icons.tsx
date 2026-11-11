/** Inline SVG chrome icons: Lucide geometry, 16 px nominal size. */

interface IconProps {
  size?: number;
  className?: string;
}

const base = (size: number): React.SVGProps<SVGSVGElement> => ({
  width: size,
  height: size,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.8,
  strokeLinecap: "round",
  strokeLinejoin: "round",
  "aria-hidden": true,
  focusable: false,
});

export function ChevronLeft({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path d="M15 18l-6-6 6-6" />
    </svg>
  );
}

export function ChevronRight({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path d="M9 18l6-6-6-6" />
    </svg>
  );
}

export function ChevronUp({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path d="M18 15l-6-6-6 6" />
    </svg>
  );
}

export function Folder({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path d="M3 8a2 2 0 0 1 2-2h3.5l2 2H19a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
    </svg>
  );
}

export function FolderOpen({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path d="M3 8a2 2 0 0 1 2-2h3.5l2 2H19a2 2 0 0 1 2 2v1" />
      <path d="M3 9.5V17a2 2 0 0 0 2 2h13.2a1.5 1.5 0 0 0 1.44-1.08L21.5 12H6.6a2 2 0 0 0-1.9 1.36z" />
    </svg>
  );
}

export function Search({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <circle cx="11" cy="11" r="7" />
      <path d="M20 20l-4.2-4.2" />
    </svg>
  );
}

export function Refresh({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path d="M21 12a9 9 0 1 1-2.64-6.36" />
      <path d="M21 3v6h-6" />
    </svg>
  );
}

export function Home({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path d="M4 10.5L12 4l8 6.5V20a1 1 0 0 1-1 1h-4v-6H9v6H5a1 1 0 0 1-1-1z" />
    </svg>
  );
}

export function Drive({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <rect x="3" y="6" width="18" height="12" rx="2.5" />
      <path d="M3 13h18" />
      <circle cx="7.5" cy="16" r="0.6" fill="currentColor" />
    </svg>
  );
}

export function Sun({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
    </svg>
  );
}

export function Moon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5z" />
    </svg>
  );
}

export function Monitor({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <rect x="3" y="4" width="18" height="12" rx="2" />
      <path d="M8 20h8M12 16v4" />
    </svg>
  );
}
