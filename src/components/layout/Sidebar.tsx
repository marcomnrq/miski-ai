import { useState } from "react";
import { NavLink } from "react-router-dom";
import { Home, Mic, FileText, MessageSquare, Settings, Moon, Sun } from "lucide-react";

const NAV_ITEMS = [
  { to: "/", label: "Home", icon: Home, end: true },
  { to: "/recorder", label: "Record", icon: Mic, end: false },
  { to: "/meetings", label: "Meetings", icon: FileText, end: false },
  { to: "/chat", label: "Chat", icon: MessageSquare, end: false },
  { to: "/settings", label: "Settings", icon: Settings, end: false },
];

export function Sidebar() {
  const [dark, setDark] = useState(() =>
    document.documentElement.classList.contains("dark")
  );

  const toggleTheme = () => {
    const next = !dark;
    setDark(next);
    document.documentElement.classList.toggle("dark", next);
  };

  return (
    <aside className="w-56 h-screen border-r border-[var(--border)] bg-[var(--surface)] flex flex-col shrink-0">
      {/* Brand */}
      <div className="p-4 border-b border-[var(--border)]">
        <h2 className="text-lg font-semibold text-[var(--text-primary)]">
          Miski AI
        </h2>
        <p className="text-xs text-[var(--text-secondary)]">
          Private meeting notes
        </p>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-2 space-y-1">
        {NAV_ITEMS.map(({ to, label, icon: Icon, end }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
                isActive
                  ? "bg-[var(--accent)] text-white"
                  : "text-[var(--text-secondary)] hover:bg-[var(--surface-raised)] hover:text-[var(--text-primary)]"
              }`
            }
          >
            <Icon size={18} />
            <span>{label}</span>
          </NavLink>
        ))}
      </nav>

      {/* Footer */}
      <div className="p-3 border-t border-[var(--border)] space-y-2">
        <button
          onClick={toggleTheme}
          className="flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-[var(--text-secondary)] hover:bg-[var(--surface-raised)] transition-colors w-full cursor-pointer"
        >
          {dark ? <Sun size={18} /> : <Moon size={18} />}
          <span>{dark ? "Light Mode" : "Dark Mode"}</span>
        </button>
        <p className="text-xs text-[var(--text-secondary)] px-3">v0.1.0</p>
      </div>
    </aside>
  );
}