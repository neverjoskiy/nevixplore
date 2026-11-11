import { Drive, Home } from "./icons";

interface SidebarProps {
  home: string;
  current: string;
  onNavigate: (path: string) => void;
}

/**
 * Places to go. At PR-0 this is the home folder plus the drive roots that the
 * webview can name without asking Rust; PR-4 replaces the roots with a real
 * `list_drives` command.
 */
export function Sidebar({ home, current, onNavigate }: SidebarProps) {
  const roots = ["C:\\", "D:\\", "E:\\"];

  return (
    <nav className="sidebar" aria-label="Places">
      <SidebarGroup title="Quick access">
        <SidebarItem
          label="Home"
          icon={<Home size={15} />}
          active={samePath(current, home)}
          onClick={() => {
            onNavigate(home);
          }}
        />
      </SidebarGroup>

      <SidebarGroup title="This PC">
        {roots.map((root) => (
          <SidebarItem
            key={root}
            label={root}
            icon={<Drive size={15} />}
            active={samePath(current, root)}
            onClick={() => {
              onNavigate(root);
            }}
          />
        ))}
      </SidebarGroup>
    </nav>
  );
}

function SidebarGroup({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="sidebar__group">
      <h2 className="sidebar__heading">{title}</h2>
      <ul className="sidebar__list">{children}</ul>
    </section>
  );
}

interface SidebarItemProps {
  label: string;
  icon: React.ReactNode;
  active: boolean;
  onClick: () => void;
}

function SidebarItem({ label, icon, active, onClick }: SidebarItemProps) {
  return (
    <li>
      <button
        className="sidebar__item"
        aria-current={active ? "page" : undefined}
        onClick={onClick}
        title={label}
      >
        <span className="sidebar__icon">{icon}</span>
        <span className="truncate">{label}</span>
      </button>
    </li>
  );
}

/** Windows paths are case-insensitive and tolerate a trailing separator. */
function samePath(a: string, b: string): boolean {
  const normalize = (value: string): string => value.replace(/[\\/]+$/, "").toLowerCase();
  return normalize(a) === normalize(b);
}
