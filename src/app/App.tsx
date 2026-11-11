import { useCallback, useEffect, useState } from "react";

import { api, isTauri, mockApi, toAppError } from "../api/client";
import type { AppError, Bootstrap, Theme } from "../api/client";
import { useAppliedTheme } from "./theme";
import { ListPane } from "../components/ListPane";
import { Sidebar } from "../components/Sidebar";
import { StatusBar } from "../components/StatusBar";
import { Toolbar } from "../components/Toolbar";
import "../styles/chrome.css";

const client = isTauri() ? api : mockApi;

/**
 * The application shell.
 *
 * Owns the boot payload, the current directory and the theme. Directory
 * listing lands in PR-4; until then the list pane shows an empty state that
 * names the folder it would read.
 */
export function App() {
  const [boot, setBoot] = useState<Bootstrap | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const [theme, setTheme] = useState<Theme>("system");
  const [path, setPath] = useState("");
  const resolvedTheme = useAppliedTheme(theme);

  useEffect(() => {
    let cancelled = false;
    client
      .bootstrap()
      .then((result) => {
        if (cancelled) return;
        setBoot(result);
        setTheme(result.settings.theme);
        setPath(result.startDir);
        document.documentElement.dataset.density = result.settings.density;
      })
      .catch((err: unknown) => {
        if (!cancelled) setError(toAppError(err));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const chooseTheme = useCallback((next: Theme) => {
    setTheme(next);
    // Optimistic: useAppliedTheme already repainted, so a failed save leaves a
    // working UI plus a warning instead of a frozen toggle.
    client.setTheme(next).catch((err: unknown) => {
      console.warn("theme not saved:", toAppError(err).message);
    });
  }, []);

  const browse = useCallback(() => {
    client
      .pickFolder(path)
      .then((picked) => {
        if (picked) setPath(picked);
      })
      .catch((err: unknown) => {
        setError(toAppError(err));
      });
  }, [path]);

  if (error) {
    return (
      <main className="boot-error" data-selectable>
        <h1>NevIxplore could not start</h1>
        <p className="boot-error__message">{error.message}</p>
        {error.detail !== error.message && <pre className="boot-error__detail">{error.detail}</pre>}
        <button
          className="button"
          onClick={() => {
            window.location.reload();
          }}
        >
          Try again
        </button>
      </main>
    );
  }

  if (!boot) {
    // Neutral chrome beats a spinner: the real paint is sub-frame, and a flash
    // of spinner reads as slowness.
    return <main className="app app--booting" aria-busy="true" />;
  }

  return (
    <main className="app">
      <Toolbar
        path={path}
        theme={theme}
        onNavigate={setPath}
        onBrowse={browse}
        onTheme={chooseTheme}
      />
      <div className="app__body">
        <Sidebar home={boot.startDir} current={path} onNavigate={setPath} />
        <ListPane path={path} onBrowse={browse} />
      </div>
      <StatusBar path={path} version={boot.version} theme={resolvedTheme} />
    </main>
  );
}
