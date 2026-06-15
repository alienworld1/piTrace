import { NavLink } from "react-router";

const navItems = [
  { label: "Cases", to: "/" },
  { label: "New case", to: "/cases/new" },
];

export function Sidebar() {
  return (
    <aside className="flex h-screen w-[280px] shrink-0 flex-col overflow-y-auto border-r border-line bg-surface px-5 py-6">
      <div className="flex items-center gap-3">
        <div className="flex h-11 w-11 items-center justify-center rounded-xl border border-cyan/35 bg-cyan/10 technical text-sm font-bold text-cyan">
          pi
        </div>
        <div>
          <p className="font-display text-xl font-semibold text-ink">piTrace</p>
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Read-only triage</p>
        </div>
      </div>

      <nav className="mt-10 space-y-2">
        {navItems.map((item) => (
          <NavLink
            className={({ isActive }) =>
              `block rounded-lg border px-4 py-3 text-sm font-semibold transition ${
                isActive
                  ? "border-cyan/50 bg-cyan/10 text-cyan"
                  : "border-transparent text-muted hover:border-line hover:bg-panel"
              }`
            }
            key={item.to}
            to={item.to}
          >
            {item.label}
          </NavLink>
        ))}
      </nav>

      <div className="mt-auto rounded-xl border border-line bg-panel p-4">
        <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Local status</p>
        <p className="mt-2 text-sm leading-6 text-muted">No files leave this device. Imports record local paths and file identity.</p>
      </div>
    </aside>
  );
}
