interface StatusBarProps {
  path: string;
  version: string;
  theme: "light" | "dark";
}

/**
 * Bottom bar. Item counts and selection totals arrive with the list (PR-4);
 * the version is here from day one so a bug report can name a build.
 */
export function StatusBar({ path, version, theme }: StatusBarProps) {
  return (
    <footer className="status-bar">
      <span className="status-bar__cell">0 items</span>
      <span className="status-bar__cell status-bar__cell--path truncate" title={path}>
        {path}
      </span>
      <span className="status-bar__spacer" />
      <span className="status-bar__cell status-bar__cell--muted" title={`Theme: ${theme}`}>
        {theme}
      </span>
      <span className="status-bar__cell status-bar__cell--muted">v{version}</span>
    </footer>
  );
}
