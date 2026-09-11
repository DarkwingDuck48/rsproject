/**
 * @fileoverview Общие константы приложения.
 *
 * ВНИМАНИЕ: `APP_VERSION` должна совпадать с версией в `app/src-tauri/Cargo.toml`
 * и `package.json` (SemVer). Держать в одном месте нельзя — Cargo.toml хранит
 * версию workspace-крейта, поэтому при бампе версии обновляй все три места.
 */

/** Ссылка на репозиторий проекта. */
export const REPO_URL = "https://github.com/DarkwingDuck48/rsproject";

/** Текущая версия приложения (SemVer). */
export const APP_VERSION = "0.1.1";
