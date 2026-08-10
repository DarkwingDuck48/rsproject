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
