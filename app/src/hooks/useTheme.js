/**
 * @fileoverview Управление темой приложения (светлая/тёмная) — задача 5.16.
 *
 * Тема переключается двумя механизмами:
 *   - атрибут `data-theme` на корневом элементе `.app` переключает
 *     CSS-переменные в App.css (цвета фона/текста/акцентов);
 *   - алгоритм antd (`theme.defaultAlgorithm` / `theme.darkAlgorithm`)
 *     переключает переменные `--ant-*`, которыми стилизуются компоненты antd
 *     (кнопки, таблицы, диалоги и т.д.).
 *
 * Выбор сохраняется в localStorage и переживает перезапуск приложения.
 */

import { useState } from "react";

/** Ключ localStorage для выбранной темы. */
export const THEME_STORAGE_KEY = "rsproject.theme";

/**
 * Читает сохранённую тему из localStorage (best-effort).
 * @returns {"light"|"dark"}
 */
function readStoredTheme() {
  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "dark") return "dark";
  } catch {
    // localStorage может быть недоступен (приватный режим и т.п.) —
    // молча используем светлую тему по умолчанию.
  }
  return "light";
}

/**
 * Хук темы.
 * @returns {{ themeMode: "light"|"dark", toggleTheme: () => void }}
 */
export function useTheme() {
  const [themeMode, setThemeMode] = useState(readStoredTheme);

  /** Переключить тему и сохранить выбор. */
  function toggleTheme() {
    const next = themeMode === "light" ? "dark" : "light";
    setThemeMode(next);
    try {
      window.localStorage.setItem(THEME_STORAGE_KEY, next);
    } catch {
      // localStorage недоступен — просто не сохраняем выбор.
    }
  }

  return { themeMode, toggleTheme };
}
