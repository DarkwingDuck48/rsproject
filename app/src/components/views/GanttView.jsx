import { Button, Card, Empty, Space, Spin, Typography, message } from "antd";
import { useEffect, useState } from "react";
import {
  getCriticalPath,
  getTaskTree,
  recalculateCriticalPath,
} from "../../lib/api";
import { isNoProjectError } from "../../lib/errors";
import { formatDate } from "../../lib/format";

/* ═══════════════════════════════════════════════════════════
   Геометрия диаграммы (пиксельные константы).
   Значения должны совпадать с CSS-переменными --gantt-*,
   которые выставляются инлайном на корневом элементе.
   ═══════════════════════════════════════════════════════════ */
const DAY_MS = 86_400_000;
const DAY_W = 36; // ширина одного дня, px
const ROW_H = 34; // высота строки задачи, px
const BAR_H = 22; // высота полосы задачи, px
const HEADER_H = 34; // высота оси времени, px
const NAME_W = 264; // ширина колонки имён, px

/** Короткие названия месяцев (для оси времени). */
const MONTHS = [
  "Янв",
  "Фев",
  "Мар",
  "Апр",
  "Май",
  "Июн",
  "Июл",
  "Авг",
  "Сен",
  "Окт",
  "Ноя",
  "Дек",
];

/**
 * RFC 3339-строка → timestamp полночи UTC.
 * Работаем в UTC, чтобы таймзона пользователя не сдвигала границы дней.
 */
function toUtcMidnight(iso) {
  const [y, m, d] = iso.slice(0, 10).split("-").map(Number);
  return Date.UTC(y, m - 1, d);
}

/** Количество дней между двумя RFC 3339-датами (для end ≥ start). */
function diffDays(fromIso, toIso) {
  return Math.round((toUtcMidnight(toIso) - toUtcMidnight(fromIso)) / DAY_MS);
}

/**
 * Дерево задач → плоский список строк диаграммы.
 * Каждая строка — TaskInfo + глубина иерархии + индекс ряда.
 * @param {TaskTreeNode[]} nodes
 * @param {number} depth
 * @param {Array<Object>} rows
 */
function flattenTree(nodes, depth = 0, rows = []) {
  for (const node of nodes) {
    rows.push({ ...node.task, depth, rowIdx: rows.length });
    flattenTree(node.children, depth + 1, rows);
  }
  return rows;
}

/**
 * Ось времени: названия месяцев + числа дней.
 * Строится отдельным SVG, чтобы её можно было «приклеить» сверху при скролле.
 */
function renderAxis({ rangeStart, totalDays }) {
  const width = totalDays * DAY_W;
  const months = [];
  const dayTexts = [];

  for (let i = 0; i < totalDays; i++) {
    const d = new Date(rangeStart + i * DAY_MS);
    const key = `${d.getUTCFullYear()}-${d.getUTCMonth()}`;
    if (!months.length || months[months.length - 1].key !== key) {
      months.push({
        key,
        startIdx: i,
        label: `${MONTHS[d.getUTCMonth()]} ${d.getUTCFullYear()}`,
      });
    }
    dayTexts.push(
      <text
        key={i}
        x={i * DAY_W + DAY_W / 2}
        y={HEADER_H - 8}
        textAnchor="middle"
        fontSize={10}
        fill="var(--gantt-text)"
      >
        {d.getUTCDate()}
      </text>,
    );
  }

  return (
    <svg
      width={width}
      height={HEADER_H}
      role="img"
      aria-label="Ось времени диаграммы Ганта"
    >
      {months.map((month, i) => {
        const next = months[i + 1];
        const endIdx = next ? next.startIdx : totalDays;
        return (
          <text
            key={month.key}
            x={((month.startIdx + endIdx) / 2) * DAY_W}
            y={15}
            textAnchor="middle"
            fontSize={11}
            fontWeight={600}
            fill="var(--gantt-text)"
          >
            {month.label}
          </text>
        );
      })}
      {dayTexts}
    </svg>
  );
}

/**
 * Полотно диаграммы: фон выходных, сетка дней, полосы задач,
 * стрелки зависимостей и заголовки-тултипы.
 */
function renderChart({
  rows,
  byId,
  criticalSet,
  rangeStart,
  rangeStartIso,
  totalDays,
}) {
  const width = totalDays * DAY_W;
  const height = rows.length * ROW_H;
  const background = [];
  const separators = [];

  // Фон выходных + вертикальная сетка каждые 7 дней
  for (let i = 0; i < totalDays; i++) {
    const dow = new Date(rangeStart + i * DAY_MS).getUTCDay();
    if (dow === 0 || dow === 6) {
      background.push(
        <rect
          key={`weekend-${i}`}
          x={i * DAY_W}
          y={0}
          width={DAY_W}
          height={height}
          fill="var(--gantt-weekend)"
        />,
      );
    }
    if (i % 7 === 0) {
      background.push(
        <line
          key={`grid-${i}`}
          x1={i * DAY_W}
          y1={0}
          x2={i * DAY_W}
          y2={height}
          stroke="var(--gantt-grid-strong)"
        />,
      );
    }
  }

  // Горизонтальные разделители строк
  rows.forEach((_, r) => {
    separators.push(
      <line
        key={`sep-${r}`}
        x1={0}
        y1={r * ROW_H}
        x2={width}
        y2={r * ROW_H}
        stroke="var(--gantt-grid)"
        strokeWidth={1}
      />,
    );
  });

  // Полосы задач: x и ширина считаются от индекса дня относительно начала диапазона.
  // Интервал [start, end] включается целиком, поэтому ширина = (end - start + 1) * DAY_W.
  const bars = rows.map((row) => {
    const startIdx = Math.max(0, diffDays(rangeStartIso, row.date_start));
    const endIdx = Math.max(startIdx, diffDays(rangeStartIso, row.date_end));
    const barW = (endIdx - startIdx + 1) * DAY_W - 2;
    const y = row.rowIdx * ROW_H + (ROW_H - BAR_H) / 2;
    const isCritical = criticalSet.has(row.id);
    const kind = row.is_summary
      ? "gantt-bar--summary"
      : isCritical
        ? "gantt-bar--critical"
        : "gantt-bar--task";
    const tip =
      `${row.name} | ${formatDate(row.date_start)} – ` +
      `${formatDate(row.date_end)} | ${row.duration_days} дн.`;
    return (
      <g key={row.id} className={`gantt-bar ${kind}`}>
        <title>{tip}</title>
        <rect
          x={startIdx * DAY_W + 1}
          y={y}
          width={barW}
          height={BAR_H}
          rx={3}
        />
      </g>
    );
  });

  // Стрелки зависимостей: от правого края предшественника к левому краю зависимой задачи.
  // Лаг в днях сдвигает начало стрелки (если задан).
  const arrows = [];
  rows.forEach((row) => {
    for (const dep of row.dependencies ?? []) {
      const pred = byId[dep.depends_on];
      if (!pred) continue;
      const x1 =
        (diffDays(rangeStartIso, pred.date_end) + 1 + (dep.lag_days ?? 0)) *
        DAY_W;
      const y1 = pred.rowIdx * ROW_H + ROW_H / 2;
      const x2 =
        Math.max(0, diffDays(rangeStartIso, row.date_start)) * DAY_W + 2;
      const y2 = row.rowIdx * ROW_H + ROW_H / 2;
      // Лаг может увести начало стрелки за правый край диаграммы —
      // клампим к границе, чтобы линия не обрезалась невидимо.
      const clampedX1 = Math.min(Math.max(x1, 0), width);
      arrows.push(
        <line
          key={`${row.id}:${dep.depends_on}`}
          x1={clampedX1}
          y1={y1}
          x2={x2}
          y2={y2}
          className="gantt-arrow"
          markerEnd="url(#gantt-arrowhead)"
        />,
      );
    }
  });

  return (
    <svg width={width} height={height} role="img" aria-label="Диаграмма Ганта">
      <defs>
        <marker
          id="gantt-arrowhead"
          viewBox="0 0 8 8"
          refX="8"
          refY="4"
          markerWidth="6"
          markerHeight="6"
          orient="auto"
        >
          <path d="M 0 0 L 8 4 L 0 8 Z" fill="var(--gantt-text)" />
        </marker>
      </defs>
      {background}
      {separators}
      {bars}
      {arrows}
    </svg>
  );
}

/**
 * Вкладка «Диаграмма Ганта»: SVG-диаграмма с подсветкой критического пути.
 *
 * @param {Object} props
 * @param {number} props.dataVersion - Счётчик изменений данных приложения.
 * @param {Function} props.onDataChange - Уведомить приложение об изменении данных.
 */
export default function GanttView({ dataVersion, onDataChange }) {
  const [rows, setRows] = useState([]);
  const [loading, setLoading] = useState(false);
  /** UUID'ы задач критического пути (null — не рассчитан). */
  const [criticalPath, setCriticalPath] = useState(null);
  const [recalcLoading, setRecalcLoading] = useState(false);

  async function load() {
    setLoading(true);
    // Данные могли измениться — кэш критического пути протух.
    setCriticalPath(null);
    try {
      const tree = await getTaskTree();
      setRows(flattenTree(tree));
      // Восстанавливаем уже посчитанный критический путь из общего стейта,
      // чтобы подсветка переживала переключение вкладок (см. 5.15).
      const path = await getCriticalPath();
      if (path) setCriticalPath(path);
    } catch (err) {
      setRows([]);
      if (!isNoProjectError(err)) {
        message.error(`Не удалось загрузить диаграмму: ${err}`);
      }
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void load();
  }, [dataVersion]);

  async function handleRecalc() {
    setRecalcLoading(true);
    try {
      const path = await recalculateCriticalPath();
      setCriticalPath(path);
      message.success(`Критический путь: ${path.length} задач`);
      // Критический путь пересчитан — пусть статус-бар и другие панели
      // перечитают данные (5.18).
      onDataChange?.();
    } catch (err) {
      message.error(`Не удалось рассчитать критический путь: ${err}`);
    } finally {
      setRecalcLoading(false);
    }
  }

  // Агрегируем диапазон дат и словарь id → строка, только если задачи есть.
  let chart = null;
  if (rows.length > 0) {
    const rangeStartIso = rows.reduce((a, b) =>
      a.date_start < b.date_start ? a : b,
    ).date_start;
    const rangeEndIso = rows.reduce((a, b) =>
      a.date_end > b.date_end ? a : b,
    ).date_end;
    chart = {
      rangeStart: toUtcMidnight(rangeStartIso),
      rangeStartIso,
      totalDays: Math.max(1, diffDays(rangeStartIso, rangeEndIso) + 1),
      byId: Object.fromEntries(rows.map((row) => [row.id, row])),
      criticalSet: new Set(criticalPath ?? []),
    };
  }

  return (
    <section className="view">
      <div
        className="gantt"
        style={{
          "--gantt-names-width": `${NAME_W}px`,
          "--gantt-chart-width": `${chart ? chart.totalDays * DAY_W : 0}px`,
          "--gantt-header-height": `${HEADER_H}px`,
          "--gantt-row-height": `${ROW_H}px`,
        }}
      >
        <Typography.Title level={3}>Диаграмма Ганта</Typography.Title>

        <Space className="views-toolbar" wrap>
          <Button
            type="dashed"
            loading={recalcLoading}
            disabled={rows.length === 0}
            onClick={handleRecalc}
          >
            Рассчитать критический путь
          </Button>
          <div className="gantt-legend">
            <span className="gantt-legend__item">
              <i className="gantt-legend__swatch gantt-legend__swatch--task" />
              Задача
            </span>
            <span className="gantt-legend__item">
              <i className="gantt-legend__swatch gantt-legend__swatch--summary" />
              Сводная
            </span>
            <span className="gantt-legend__item">
              <i className="gantt-legend__swatch gantt-legend__swatch--critical" />
              Критический путь
            </span>
          </div>
        </Space>

        {loading ? (
          <div className="views-loading">
            <Spin />
          </div>
        ) : rows.length === 0 ? (
          <Card>
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description="Задач нет. Добавьте задачи во вкладке «Задачи»."
            />
          </Card>
        ) : (
          <div className="gantt-scroll">
            <div className="gantt-grid">
              <div className="gantt-corner" />
              <div className="gantt-axis">
                {renderAxis({
                  rangeStart: chart.rangeStart,
                  totalDays: chart.totalDays,
                })}
              </div>
              <div className="gantt-names">
                {rows.map((row) => (
                  <div
                    key={row.id}
                    className={`gantt-name${
                      row.is_summary ? " gantt-name--summary" : ""
                    }`}
                    style={{ paddingLeft: 8 + row.depth * 16 }}
                    title={row.name}
                  >
                    {row.is_summary ? "▸ " : ""}
                    {row.name}
                  </div>
                ))}
              </div>
              <div className="gantt-chart">
                {renderChart({
                  rows,
                  byId: chart.byId,
                  criticalSet: chart.criticalSet,
                  rangeStart: chart.rangeStart,
                  rangeStartIso: chart.rangeStartIso,
                  totalDays: chart.totalDays,
                })}
              </div>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
