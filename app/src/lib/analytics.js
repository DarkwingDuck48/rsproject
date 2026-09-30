/**
 * @fileoverview Аналитика по проекту, считаемая на фронтенде.
 * Общие хелперы для вьюх и статус-бара — чтобы логика была в одном месте.
 */

import { inclusiveDays } from "./format";

/**
 * Сводка по критическому пути: длительность, число задач, имена первых.
 * Единая реализация для статус-бара (нужны только `days`) и вкладки
 * «Проект» (нужны `days`, `count`, `names`, `more`).
 *
 * Длительность пути — промежуток от самого раннего начала до самого позднего
 * конца критических задач (включает лаги). У сводных задач нет дат — их из
 * длительности исключаем (как в статус-баре).
 *
 * @param {?string[]} criticalPath - UUID'ы задач пути (или null, если не считали).
 * @param {TaskInfo[]} tasks - Плоский список задач (для имён и дат).
 * @param {number} [maxNames=4] - Сколько имён задач вернуть в `names`.
 * @returns {?{ days: number, count: number, names: string[], more: number }}
 *          null, если путь не рассчитан или задач с датами на пути нет.
 */
export function criticalPathSpan(criticalPath, tasks, maxNames = 4) {
  if (!criticalPath || criticalPath.length === 0) return null;

  const byId = new Map(tasks.map((task) => [task.id, task]));
  const pathTasks = criticalPath
    .map((id) => byId.get(id))
    .filter((task) => task && task.date_start && task.date_end);
  if (pathTasks.length === 0) return null;

  const start = pathTasks.reduce((a, b) =>
    a.date_start < b.date_start ? a : b,
  ).date_start;
  const end = pathTasks.reduce((a, b) =>
    a.date_end > b.date_end ? a : b,
  ).date_end;

  return {
    days: inclusiveDays(start, end),
    count: pathTasks.length,
    names: pathTasks.slice(0, maxNames).map((task) => task.name),
    more: Math.max(0, pathTasks.length - maxNames),
  };
}
