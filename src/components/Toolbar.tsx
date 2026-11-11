import { useRef, useState } from "react";

import { ChevronLeft, FolderOpen, Monitor, Moon, Refresh, Search, Sun } from "./icons";
import type { Theme } from "../api/client";

interface ToolbarProps {
  path: string;
  theme: Theme;
  onNavigate: (path: string) => void;
  onBrowse: () => void;
  onTheme: (theme: Theme) => void;
}

const THEMES: readonly { value: Theme; label: string; Icon: typeof Sun }[] = [
  { value: "system", label: "System theme", Icon: Monitor },
  { value: "light", label: "Light theme", Icon: Sun },
  { value: "dark", label: "Dark theme", Icon: Moon },
];

/**
 * Top chrome: navigation buttons, the address bar, search, and the theme
 * switch. Search is a placeholder input until PR-8 wires fuzzy matching.
 */
export function Toolbar({ path, theme, onNavigate, onBrowse, onTheme }: ToolbarProps) {
  return (
    <header className="toolbar">
      <div className="toolbar__nav">
        <button className="icon-button" title="Back" aria-label="Back" disabled>
          <ChevronLeft />
        </button>
        <button
          className="icon-button"
          title="Choose folder"
          aria-label="Choose folder"
          onClick={onBrowse}
        >
          <FolderOpen size={15} />
        </button>
      </div>

      <AddressBar path={path} onNavigate={onNavigate} onBrowse={onBrowse} />

      <div className="toolbar__actions">
        <label className="search-field">
          <Search size={14} />
          <input
            type="search"
            placeholder="Search this folder"
            aria-label="Search this folder"
            spellCheck={false}
          />
        </label>

        <button
          className="icon-button"
          title="Refresh"
          aria-label="Refresh"
          onClick={() => {
            onNavigate(path);
          }}
        >
          <Refresh size={15} />
        </button>

        <div className="theme-switch" role="group" aria-label="Theme">
          {THEMES.map(({ value, label, Icon }) => (
            <button
              key={value}
              className="theme-switch__option"
              aria-pressed={theme === value}
              aria-label={label}
              title={label}
              onClick={() => {
                onTheme(value);
              }}
            >
              <Icon size={14} />
            </button>
          ))}
        </div>
      </div>
    </header>
  );
}

interface AddressBarProps {
  path: string;
  onNavigate: (path: string) => void;
  onBrowse: () => void;
}

/**
 * Shows the path as editable text. Clicking selects the whole path so typing
 * replaces it, which is the behaviour Explorer users expect; Enter commits and
 * Escape reverts.
 */
export function AddressBar({ path, onNavigate, onBrowse }: AddressBarProps) {
  // `null` means "not editing", so the field always shows the live path and
  // there is no prop-to-state mirror to keep in sync.
  const [draft, setDraft] = useState<string | null>(null);
  const shown = draft ?? path;
  const input = useRef<HTMLInputElement>(null);

  const commit = (): void => {
    const trimmed = (draft ?? "").trim();
    setDraft(null);
    if (trimmed.length > 0 && trimmed !== path) {
      onNavigate(trimmed);
    }
  };

  return (
    <div className="address-bar">
      <input
        ref={input}
        className="address-bar__input"
        value={shown}
        aria-label="Folder path"
        spellCheck={false}
        autoComplete="off"
        onChange={(event) => {
          setDraft(event.target.value);
        }}
        onFocus={(event) => {
          event.target.select();
        }}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            commit();
            event.currentTarget.blur();
          } else if (event.key === "Escape") {
            setDraft(null);
            event.currentTarget.blur();
          }
        }}
      />
      <button
        className="address-bar__browse"
        onClick={onBrowse}
        aria-label="Browse for a folder"
        title="Browse for a folder"
      >
        <FolderOpen size={15} />
      </button>
    </div>
  );
}
