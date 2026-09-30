/**
 * @fileoverview Общие функции форматирования для отображения данных.
 */

import dayjs from "dayjs";

/** Миллисекунд в дне (для расчёта длительности интервалов). */
const DAY_MS = 86_400_000;

/**
 * Форматирует дату из бэкенда (RFC 3339, напр. "2026-09-11T00:00:00Z")
 * в удобный для отображения вид "DD.MM.YYYY".
 *
 * БЕРЁМ ТОЛЬКО дату-часть строки (первые 10 символов) без перевода в локальную
 * таймзону: даты на бэкенде — полночь UTC, а dayjs-форматирование в локальном
 * часовом поясе могло бы сдвинуть день назад в отрицательных зонах.
 *
 * @param {?string} iso - Дата от бэкенда или null (у сводных задач нет дат).
 * @returns {string}
 */
export function formatDate(iso) {
  if (!iso) return "—";
  return iso.slice(0, 10).split("-").reverse().join(".");
}

/**
 * Русские формы множественного числа для дней ("1 день", "2 дня", "5 дней").
 * @param {number} n - Количество дней (duration_days).
 * @returns {string}
 */
export function pluralDays(n) {
  const mod10 = n % 10;
  const mod100 = n % 100;
  if (mod100 >= 11 && mod100 <= 14) return `${n} дней`;
  if (mod10 === 1) return `${n} день`;
  if (mod10 >= 2 && mod10 <= 4) return `${n} дня`;
  return `${n} дней`;
}

/**
 * Длительность интервала [startIso, endIso] включительно, в днях.
 * Используется для длительности критического пути (статус-бар, 5.18)
 * и аналитики вкладки «Проект».
 * @param {string} startIso - Начало (RFC 3339 или "YYYY-MM-DD")
 * @param {string} endIso   - Конец (RFC 3339 или "YYYY-MM-DD")
 * @returns {number}
 */
export function inclusiveDays(startIso, endIso) {
  const [y1, m1, d1] = startIso.slice(0, 10).split("-").map(Number);
  const [y2, m2, d2] = endIso.slice(0, 10).split("-").map(Number);
  return (
    Math.round((Date.UTC(y2, m2 - 1, d2) - Date.UTC(y1, m1 - 1, d1)) / DAY_MS) +
    1
  );
}

/**
 * Форматирует JavaScript Date в "ДД.ММ.ГГГГ ЧЧ:ММ" для статус-бара (5.18).
 * На входе — локальный объект Date (в отличие от formatDate, где приходят
 * строки с бэкенда), поэтому используем dayjs в локальном часовом поясе.
 * @param {?Date} date - Момент времени или null (если ещё не сохраняли).
 * @returns {string}
 */
export function formatDateTime(date) {
  if (!date) return "—";
  return dayjs(date).format("DD.MM.YYYY HH:mm");
}

const RUB_FORMATTER = Intl.NumberFormat("ru-RU", {
  style: "currency",
  currency: "RUB",
});

/**
 * Форматирует число в сумму в рублях ("1 234,56 ₽").
 * ФОРМАТТЕР создаётся один раз на модуль — Intl.NumberFormat тяжёлый,
 * на каждом рендере плодить его не стоит.
 * @param {?number} number - Сумма от бэкенда или null/undefined.
 * @returns {string}
 */
export function formatNumber(number) {
  if (number === null || number === undefined) return "-";
  return RUB_FORMATTER.format(number);
}
