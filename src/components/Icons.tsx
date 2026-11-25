import {
  Check,
  Clock,
  Download,
  ExternalLink,
  Film,
  Folder,
  Music,
  RefreshCw,
  Settings,
  SlidersHorizontal,
  Trash2,
  TriangleAlert,
  X,
} from "lucide-react";

interface IconProps {
  size?: number;
  className?: string;
}

/** Lucide is the most used icon set (570M downloads/month in 2025-2026). */
const STROKE = 1.75;

export function IconDownload({ size = 16, className }: IconProps) {
  return <Download size={size} strokeWidth={STROKE} className={className} />;
}

export function IconFilm({ size = 16, className }: IconProps) {
  return <Film size={size} strokeWidth={STROKE} className={className} />;
}

export function IconMusic({ size = 16, className }: IconProps) {
  return <Music size={size} strokeWidth={STROKE} className={className} />;
}

export function IconSliders({ size = 16, className }: IconProps) {
  return <SlidersHorizontal size={size} strokeWidth={STROKE} className={className} />;
}

export function IconSettings({ size = 16, className }: IconProps) {
  return <Settings size={size} strokeWidth={STROKE} className={className} />;
}

export function IconFolder({ size = 16, className }: IconProps) {
  return <Folder size={size} strokeWidth={STROKE} className={className} />;
}

export function IconX({ size = 16, className }: IconProps) {
  return <X size={size} strokeWidth={STROKE} className={className} />;
}

export function IconCheck({ size = 16, className }: IconProps) {
  return <Check size={size} strokeWidth={STROKE} className={className} />;
}

export function IconAlert({ size = 16, className }: IconProps) {
  return <TriangleAlert size={size} strokeWidth={STROKE} className={className} />;
}

export function IconRefresh({ size = 16, className }: IconProps) {
  return <RefreshCw size={size} strokeWidth={STROKE} className={className} />;
}

export function IconTrash({ size = 16, className }: IconProps) {
  return <Trash2 size={size} strokeWidth={STROKE} className={className} />;
}

export function IconExternal({ size = 16, className }: IconProps) {
  return <ExternalLink size={size} strokeWidth={STROKE} className={className} />;
}

export function IconClock({ size = 14, className }: IconProps) {
  return <Clock size={size} strokeWidth={STROKE} className={className} />;
}

/** Flat product mark: three narrowing bars, a download built like a meter scale. */
export function IconLogo({ size = 22, className }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
      className={className}
      aria-hidden
    >
      <rect x="3" y="3.5" width="18" height="2.6" rx="0.8" />
      <rect x="6.2" y="9.2" width="11.6" height="2.6" rx="0.8" />
      <rect x="9.4" y="14.9" width="5.2" height="2.6" rx="0.8" />
    </svg>
  );
}
