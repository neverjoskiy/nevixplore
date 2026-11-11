/**
 * Theme resolution.
 *
 * "system" follows the OS via `prefers-color-scheme`, which the webview maps
 * from the Windows app theme. The resolved value is written to `data-theme` on
 * <html>; every colour in the stylesheet reads that attribute, so a theme
 * switch is one attribute write and no re-render of the tree.
 */

import { useEffect, useState } from "react";

import type { Theme } from "../api/client";

export type ResolvedTheme = "light" | "dark";

const QUERY = "(prefers-color-scheme: dark)";

/** Reads the OS preference. Returns "light" where `matchMedia` is absent. */
export function systemTheme(): ResolvedTheme {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return "light";
  }
  return window.matchMedia(QUERY).matches ? "dark" : "light";
}

/**
 * Applies the theme to the document and tracks the OS while the choice is
 * "system". Returns the value actually in effect.
 */
export function useAppliedTheme(theme: Theme): ResolvedTheme {
  const [osTheme, setOsTheme] = useState<ResolvedTheme>(systemTheme);
  const resolved = theme === "system" ? osTheme : theme;

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return undefined;
    }
    const media = window.matchMedia(QUERY);
    const handler = (event: MediaQueryListEvent): void => {
      setOsTheme(event.matches ? "dark" : "light");
    };
    media.addEventListener("change", handler);
    return () => {
      media.removeEventListener("change", handler);
    };
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = resolved;
  }, [resolved]);

  return resolved;
}
