/**
 * @fileoverview Обёртки над `invoke()` — единая точка доступа к Tauri-командам.
 *
 * ВАЖНО про имена аргументов (Tauri v2):
 * 1. Верхнеуровневые ключи invoke-объекта должны быть в `camelCase` —
 *    макрос `#[tauri::command]` конвертирует snake_case-имена параметров
 *    в lowerCamelCase и ищет ключ именно в таком виде
 *    (например, аргумент `date_start` → ключ `dateStart`).
 * 2. Поля вложенных структур (DTO), которые десериализуются через serde,
 *    НЕ конвертируются: они должны совпадать с именами полей Rust-структур
 *    (`TaskUpdateDto.date_start`, `ResourceUpdateDto.rate_measure` и т.д.).
 */

import { invoke } from "@tauri-apps/api/core";

// ═══════════════════════════════════════════════════════════
// Проект
// ═══════════════════════════════════════════════════════════

/** @returns {Promise<ProjectInfo>} */
export function getProjectInfo() {
  return invoke("get_project_info", { projectId: null });
}

/**
 * @param {string} name
 * @param {string} description
 * @param {string} dateStart - "YYYY-MM-DD"
 * @param {string} dateEnd   - "YYYY-MM-DD"
 */
export function createProject(name, description, dateStart, dateEnd) {
  return invoke("create_project", {
    name,
    description,
    dateStart,
    dateEnd,
  });
}

/**
 * @param {string} name
 * @param {string} description
 * @param {string} dateStart - "YYYY-MM-DD"
 * @param {string} dateEnd   - "YYYY-MM-DD"
 */
export function editProject(name, description, dateStart, dateEnd) {
  return invoke("edit_project", {
    name,
    description,
    dateStart,
    dateEnd,
  });
}

/** Открывает системный диалог выбора файла (бэкенд). */
export function openProject() {
  return invoke("open_project");
}

/** Открывает системный диалог сохранения файла (бэкенд). */
export function saveProject() {
  return invoke("save_project");
}

/** Закрывает текущий проект (контейнер очищается). */
export function closeProject() {
  return invoke("close_project");
}

// ═══════════════════════════════════════════════════════════
// Задачи
// ═══════════════════════════════════════════════════════════

/** @returns {Promise<TaskInfo[]>} */
export function getTasks() {
  return invoke("get_tasks", { projectId: null });
}

/** @returns {Promise<TaskTreeNode[]>} */
export function getTaskTree() {
  return invoke("get_task_tree", { projectId: null });
}

/** @returns {Promise<TaskDetailInfo>} */
export function getTask(taskId) {
  return invoke("get_task", { taskId });
}

/**
 * @param {string}   name
 * @param {?string}  dateStart - "YYYY-MM-DD" или null (для сводной задачи)
 * @param {?string}  dateEnd   - "YYYY-MM-DD" или null
 * @param {boolean}  isSummary
 * @param {?string}  parentId  - UUID родителя или null
 */
export function addTask(name, dateStart, dateEnd, isSummary, parentId) {
  return invoke("add_task", {
    name,
    dateStart,
    dateEnd,
    isSummary,
    parentId,
  });
}

/**
 * Обновление задачи. Все поля опциональны (null = не менять).
 * Поля DTO (`task_update`) — snake_case, т.к. десериализуются serde
 * по именам полей `TaskUpdateDto`.
 *
 * `parent_id` нельзя сбросить в корень (ограничение сериализации
 * `Option<Option<Uuid>>` на бэкенде) — передаём только новый родитель.
 *
 * @param {string}   taskId
 * @param {Object}   update
 * @param {?string}  update.name
 * @param {?string}  update.dateStart
 * @param {?string}  update.dateEnd
 * @param {?string}  update.status     - TaskStatus
 * @param {?string}  update.parentId   - UUID нового родителя
 */
export function editTask(taskId, update) {
  return invoke("edit_task", {
    taskId,
    task_update: {
      name: update.name,
      date_start: update.dateStart,
      date_end: update.dateEnd,
      status: update.status,
      parent_id: update.parentId,
    },
  });
}

export function deleteTask(taskId) {
  return invoke("delete_task", { taskId });
}

/**
 * @param {string}   taskId
 * @param {string}   dependsOn   - UUID задачи-предшественника
 * @param {string}   depType     - DependencyType
 * @param {?number}  lagDays     - лаг в днях или null
 */
export function addDependency(taskId, dependsOn, depType, lagDays) {
  return invoke("add_dependency", {
    taskId,
    dependsOn,
    depType,
    lagDays,
  });
}

/**
 * @param {string} taskId
 * @param {string} dependsOn - UUID задачи-предшественника
 */
export function removeDependency(taskId, dependsOn) {
  return invoke("remove_dependency", {
    taskId,
    dependsOn,
  });
}

/**
 * Назначение ресурса на задачу.
 *
 * @param {string}   taskId
 * @param {string}   resourceId
 * @param {number}   engagement - доля занятости 0.0–1.0
 * @param {?string}  dateStart  - начало окна ("YYYY-MM-DD") или null (вся задача)
 * @param {?string}  dateEnd    - конец окна или null
 */
export function assignResource(
  taskId,
  resourceId,
  engagement,
  dateStart,
  dateEnd,
) {
  return invoke("assign_resource", {
    taskId,
    resourceId,
    engagement,
    dateStart,
    dateEnd,
  });
}

export function removeAssignment(taskId, allocationId) {
  return invoke("remove_assignment", {
    taskId,
    allocationId,
  });
}

/** @returns {Promise<string[]>} UUID'ы задач критического пути. */
export function recalculateCriticalPath() {
  return invoke("recalculate_critical_path", { projectId: null });
}

/**
 * Возвращает уже посчитанный критический путь из общего стейта
 * (null, если он ещё не считался или данные менялись после расчёта).
 * @returns {Promise<?string[]>}
 */
export function getCriticalPath() {
  return invoke("get_critical_path", { projectId: null });
}

// ═══════════════════════════════════════════════════════════
// Ресурсы
// ═══════════════════════════════════════════════════════════

/** @returns {Promise<ResourceInfo[]>} */
export function getResources() {
  return invoke("get_resources");
}

/**
 * @param {string} name
 * @param {number} rate        - ставка, > 0
 * @param {string} rateMeasure - RateMeasure
 */
export function addResource(name, rate, rateMeasure) {
  return invoke("add_resource", {
    name,
    rate,
    rateMeasure,
  });
}

/**
 * @param {string} resourceId
 * @param {Object} update
 * @param {string} update.name
 * @param {number} update.rate
 * @param {string} update.rateMeasure
 */
export function editResource(resourceId, update) {
  return invoke("edit_resource", {
    resourceId,
    resource_update: {
      name: update.name,
      rate: update.rate,
      rate_measure: update.rateMeasure,
    },
  });
}

export function deleteResource(resourceId) {
  return invoke("delete_resource", { resourceId });
}

/**
 * @param {string} resourceId
 * @param {string} dateStart     - "YYYY-MM-DD"
 * @param {string} dateEnd       - "YYYY-MM-DD"
 * @param {string} exceptionType - ExceptionType
 */
export function addUnavailablePeriod(
  resourceId,
  dateStart,
  dateEnd,
  exceptionType,
) {
  return invoke("add_unavailable_period", {
    resourceId,
    dateStart,
    dateEnd,
    exceptionType,
  });
}
