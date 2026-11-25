import type { ReactNode } from "react";
import { motion } from "motion/react";

import { useI18n } from "../lib/i18n";
import { IconDownload, IconSettings, IconSliders } from "./Icons";

export type Page = "downloads" | "converter" | "settings";

interface SidebarProps {
  page: Page;
  onNavigate: (page: Page) => void;

  activeCount: number;
}

export function Sidebar({ page, onNavigate, activeCount }: SidebarProps) {
  const { t } = useI18n();

  const items: Array<{ id: Page; label: string; icon: ReactNode }> = [
    { id: "downloads", label: t("nav.downloads"), icon: <IconDownload size={20} /> },
    { id: "converter", label: t("nav.converter"), icon: <IconSliders size={20} /> },
    { id: "settings", label: t("nav.settings"), icon: <IconSettings size={20} /> },
  ];

  return (
    <aside className="sidebar">
      <nav className="nav">
        {items.map((item) => {
          const active = page === item.id;
          return (
            <button
              key={item.id}
              type="button"
              className={`nav-item ${active ? "active" : ""}`}
              onClick={() => onNavigate(item.id)}
            >
              {active && (
                <motion.span
                  layoutId="nav-rail"
                  className="nav-rail"
                  transition={{ duration: 0.18, ease: [0.2, 0, 0, 1] }}
                />
              )}
              <span className="nav-icon">{item.icon}</span>
              <span>{item.label}</span>
              {item.id === "downloads" && activeCount > 0 && (
                <span className="nav-badge mono">{activeCount}</span>
              )}
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
