/**
 * @fileoverview Русские лейблы и опции для Select по enum'ам из `logic`.
 *
 * Значения (`value`) должны совпадать с именами вариантов Rust-enum'ов,
 * которые приходят с бэкенда как строки (см. `types/models.js`).
 */

/** Лейблы меры ставки ресурса (@type {Record<RateMeasure, string>}). */
export const RATE_MEASURE_LABELS = {
  Daily: "В день",
  Hourly: "В час",
  Monthly: "В месяц",
};

/** Лейблы типов исключений (периодов недоступности) (@type {Record<ExceptionType, string>}). */
export const EXCEPTION_TYPE_LABELS = {
  Vacation: "Отпуск",
  SickLeave: "Болезнь",
  PersonalDay: "Личный день",
  Overtime: "Сверхурочные",
};

/** Лейблы статусов задачи (@type {Record<TaskStatus, string>}). */
export const TASK_STATUS_LABELS = {
  New: "Новая",
  Wait: "Ожидание",
  Processed: "В работе",
  Complete: "Завершена",
  Rejected: "Отклонена",
  Closed: "Закрыта",
};

/** Лейблы типов зависимости (@type {Record<DependencyType, string>}). */
export const DEPENDENCY_TYPE_LABELS = {
  Blocking: "Блокирующая",
  NonBlocking: "Неблокирующая",
};

/**
 * Преобразует карту лейблов в опции Select: [{ value, label }].
 * @param {Record<string, string>} labels
 * @returns {{value: string, label: string}[]}
 */
export function toOptions(labels) {
  return Object.entries(labels).map(([value, label]) => ({ value, label }));
}

/** Опции меры ставки. */
export const RATE_MEASURE_OPTIONS = toOptions(RATE_MEASURE_LABELS);

/** Опции типов исключений. */
export const EXCEPTION_TYPE_OPTIONS = toOptions(EXCEPTION_TYPE_LABELS);

/** Опции статусов задачи. */
export const TASK_STATUS_OPTIONS = toOptions(TASK_STATUS_LABELS);

/** Опции типов зависимости. */
export const DEPENDENCY_TYPE_OPTIONS = toOptions(DEPENDENCY_TYPE_LABELS);
