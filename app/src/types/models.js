/**
 * @fileoverview JSDoc-типы структур данных из крейта `logic`.
 *
 * Все типы отражают serde JSON-вывод Rust-структур, который
 * приходит на фронтенд через `invoke()` (Tauri commands).
 *
 * Соответствие Rust → JSON:
 *   Uuid          → string
 *   DateTime<Utc> → string (ISO 8601)
 *   TimeDelta     → {secs: number, nanos: number}
 *   NaiveDate     → string (ISO 8601, только дата)
 *   Weekday       → string ("Mon"|"Tue"|…)
 *   Option<T>     → T | null
 *   Vec<T>        → T[]
 *   HashMap<K,V>  → Object.<string, V>
 *   HashSet<T>    → T[]
 *   Rust enum     → string (имя варианта)
 */

// ═══════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════

/**
 * Статус задачи.
 * @typedef {'New'|'Wait'|'Processed'|'Complete'|'Rejected'|'Closed'} TaskStatus
 */

/**
 * Мера ставки ресурса.
 * @typedef {'Daily'|'Hourly'|'Monthly'} RateMeasure
 */

/**
 * Тип зависимости между задачами.
 * @typedef {'Blocking'|'NonBlocking'} DependencyType
 */

/**
 * Тип исключения (периода недоступности) ресурса.
 * @typedef {'Vacation'|'SickLeave'|'PersonalDay'|'Overtime'} ExceptionType
 */

// ═══════════════════════════════════════════════════════════
// Вспомогательные структуры
// ═══════════════════════════════════════════════════════════

/**
 * Временное окно — период с датой начала и окончания.
 * @typedef  {Object} TimeWindow
 * @property {string} date_start - Начало периода (ISO 8601)
 * @property {string} date_end   - Конец периода (ISO 8601)
 */

/**
 * Зависимость задачи от другой задачи (предшественника).
 * @typedef  {Object}         Dependency
 * @property {DependencyType} dependency_type - Тип зависимости
 * @property {string}         depends_on      - UUID задачи-предшественника
 * @property {?Duration}      lag             - Лаг/запас времени (null если отсутствует)
 */

/**
 * Период исключения — когда ресурс недоступен или работает сверхурочно.
 * @typedef  {Object}        ExceptionPeriod
 * @property {TimeWindow}    period         - Временной промежуток
 * @property {ExceptionType} exception_type - Причина исключения
 */

/**
 * Длительность (сериализация chrono::TimeDelta).
 * @typedef  {Object} Duration
 * @property {number} secs  - Секунды
 * @property {number} nanos - Наносекунды (0..999_999_999)
 */

/**
 * Календарь проекта — рабочие дни, праздники, часы в дне.
 * @typedef  {Object}   ProjectCalendar
 * @property {string[]} working_days         - Рабочие дни недели ("Mon"|"Tue"|…)
 * @property {string[]} holidays             - Праздничные даты (ISO 8601, только дата)
 * @property {number}   working_hours_per_day - Рабочих часов в дне (по умолчанию 8)
 */

// ═══════════════════════════════════════════════════════════
// Основные структуры
// ═══════════════════════════════════════════════════════════

/**
 * Проект — главная сущность приложения.
 * @typedef  {Object}                  Project
 * @property {string}                  id          - UUID проекта
 * @property {string}                  name        - Название
 * @property {string}                  description - Описание
 * @property {ProjectCalendar}         calendar    - Календарь проекта
 * @property {string}                  date_start  - Дата начала (ISO 8601)
 * @property {string}                  date_end    - Дата окончания (ISO 8601)
 * @property {Duration}                duration    - Длительность
 * @property {Object.<string, Task>}   tasks       - Задачи (ключ — UUID задачи)
 */

/**
 * Информация о проекте для вкладки «Проект» (DTO `ProjectInfo` из `commands/dto.rs`).
 * Возвращается командой `get_project_info`.
 * @typedef  {Object}       ProjectInfo
 * @property {string}       id            - UUID проекта
 * @property {string}       name          - Название
 * @property {string}       description   - Описание
 * @property {string}       date_start    - Дата начала (RFC 3339, напр. "2026-09-11T00:00:00Z")
 * @property {string}       date_end      - Дата окончания (RFC 3339)
 * @property {number}       duration_days - Длительность в днях
 */

/**
 * Информация о задаче (DTO `TaskInfo` из `dto/task.rs`).
 * Возвращается командами `get_tasks` / `get_task_tree`.
 * @typedef  {Object}       TaskInfo
 * @property {string}       id               - UUID задачи
 * @property {string}       name             - Название
 * @property {string}       date_start       - Дата начала (RFC 3339)
 * @property {string}       date_end         - Дата окончания (RFC 3339)
 * @property {number}       duration_days    - Длительность в днях
 * @property {TaskStatus}   status           - Статус
 * @property {boolean}      is_summary       - Сводная (группирующая) задача
 * @property {?string}      parent_id        - UUID родительской задачи (null для корневых)
 * @property {number}       cost             - Стоимость (по назначенным ресурсам)
 * @property {TaskDependencyInfo[]} dependencies - Зависимости (предшественники)
 * @property {number}       allocations_count - Количество назначенных ресурсов
 */

/**
 * Узел дерева задач (DTO `TaskTreeNode`).
 * @typedef  {Object}      TaskTreeNode
 * @property {TaskInfo}          task     - Задача
 * @property {TaskTreeNode[]}    children - Дочерние задачи
 */

/**
 * Информация о зависимости задачи (DTO `TaskDependencyInfo`).
 * @typedef  {Object}        TaskDependencyInfo
 * @property {string}        depends_on      - UUID задачи-предшественника
 * @property {DependencyType} dependency_type - Тип зависимости
 * @property {?number}       lag_days        - Лаг в днях (null если отсутствует)
 */

/**
 * Назначение ресурса на задачу (DTO `TaskAllocationInfo`).
 * @typedef  {Object}    TaskAllocationInfo
 * @property {string}    allocation_id   - UUID аллокации
 * @property {string}    resource_id     - UUID ресурса
 * @property {number}    engagement_rate - Доля занятости 0.0–1.0
 * @property {TimeWindow} time_window     - Временное окно занятости
 */

/**
 * Детальная информация о задаче (DTO `TaskDetailInfo`).
 * @typedef  {Object}           TaskDetailInfo
 * @property {TaskInfo}              task        - Задача
 * @property {TaskAllocationInfo[]}  allocations - Назначенные ресурсы
 */

/**
 * Информация о периоде недоступности ресурса (DTO `ExceptionPeriodInfo`).
 * @typedef  {Object}        ExceptionPeriodInfo
 * @property {TimeWindow}    period         - Период (RFC 3339 даты)
 * @property {ExceptionType} exception_type - Причина
 */

/**
 * Информация о ресурсе (DTO `ResourceInfo` из `dto/resources.rs`).
 * @typedef  {Object}                  ResourceInfo
 * @property {string}                  id                  - UUID ресурса
 * @property {string}                  name                - Название
 * @property {number}                  rate                - Ставка
 * @property {RateMeasure}             rate_measure        - Мера ставки
 * @property {ExceptionPeriodInfo[]}   unavailable_periods - Периоды недоступности
 */

/**
 * Задача.
 * @typedef  {Object}       Task
 * @property {string}       id                   - UUID задачи
 * @property {string}       name                 - Название
 * @property {string}       date_start           - Дата начала (ISO 8601)
 * @property {string}       date_end             - Дата окончания (ISO 8601)
 * @property {Duration}     duration             - Длительность
 * @property {TaskStatus}   status               - Статус
 * @property {string[]}     resource_allocations - UUID'ы назначенных ресурсов
 * @property {Dependency[]} dependencies         - Зависимости (предшественники)
 * @property {?string}      parent_id            - UUID группирующей задачи (null для корневых)
 * @property {boolean}      is_summary           - Является ли группирующей (summary)
 */

/**
 * Ресурс (глобальный, может использоваться в нескольких проектах).
 * @typedef  {Object}            Resource
 * @property {string}            id                  - UUID ресурса
 * @property {string}            name                - Название
 * @property {number}            rate                - Ставка
 * @property {RateMeasure}       rate_measure        - Мера ставки
 * @property {ExceptionPeriod[]} unavailable_periods - Периоды недоступности
 */
