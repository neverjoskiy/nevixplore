/**
 * The only module that talks to Rust.
 *
 * Every command is declared once here with its DTO shape, so a Rust rename
 * breaks one file and the compiler names the call sites. PR-0 ships the three
 * commands that exist; later PRs add their own wrappers below.
 */

export type Theme = "system" | "light" | "dark";
export type Density = "comfortable" | "compact";

export interface Settings {
  theme: Theme;
  accent: string | null;
  density: Density;
  sidebarWidth: number;
  showSidebar: boolean;
  showPreview: boolean;
  previewEnabled: boolean;
}

export interface Bootstrap {
  version: string;
  settings: Settings;
  startDir: string;
}

export interface AppError {
  kind: string;
  message: string;
  detail: string;
  path: string | null;
  code: number | null;
}

/** Commands the backend exposes. Kept in sync with `commands/mod.rs`. */
const COMMANDS = {
  bootstrap: "bootstrap",
  setTheme: "set_theme",
  pickFolder: "pick_folder",
} as const;

/**
 * True when running inside the Tauri webview. `npm run dev` in a plain browser
 * has no IPC bridge, and the shell should still render.
 */
export const isTauri = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: call } = await import("@tauri-apps/api/core");
  return call<T>(command, args);
}

/** Turns an unknown rejection into the DTO shape the UI understands. */
export function toAppError(err: unknown): AppError {
  if (typeof err === "object" && err !== null && "message" in err && "kind" in err) {
    return err as AppError;
  }
  const message = err instanceof Error ? err.message : String(err);
  return { kind: "internal", message, detail: message, path: null, code: null };
}

export const api = {
  bootstrap: (): Promise<Bootstrap> => invoke<Bootstrap>(COMMANDS.bootstrap),

  setTheme: (theme: Theme): Promise<Settings> => invoke<Settings>(COMMANDS.setTheme, { theme }),

  pickFolder: (startDir: string): Promise<string | null> =>
    invoke<string | null>(COMMANDS.pickFolder, { startDir }),
};

/** Stand-in used when the frontend runs in a browser without the Rust shell. */
export const mockApi = {
  bootstrap: (): Promise<Bootstrap> =>
    Promise.resolve({
      version: "0.1.0-web",
      settings: {
        theme: "system",
        accent: null,
        density: "comfortable",
        sidebarWidth: 220,
        showSidebar: true,
        showPreview: false,
        previewEnabled: true,
      },
      startDir: "C:\\",
    }),
  setTheme: (theme: Theme): Promise<Settings> =>
    mockApi.bootstrap().then((b) => ({ ...b.settings, theme })),
  pickFolder: (): Promise<string | null> => Promise.resolve(null),
};

export type Api = typeof api;
