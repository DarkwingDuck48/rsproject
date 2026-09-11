import {
  Button,
  Card,
  Descriptions,
  Empty,
  Space,
  Spin,
  Typography,
  message,
} from "antd";
import { EditOutlined } from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";
import dayjs from "dayjs";
import { useEffect, useState } from "react";
import EditProjectDialog from "../dialogs/EditProjectDialog";

/** Сообщения об ошибках, означающие «проект сейчас не выбран» (штатная ситуация). */
const NO_PROJECT_ERRORS = ["Не выбран проект", "Проект не найден"];

/**
 * Форматирует дату проекта (RFC 3339, напр. "2026-09-11T00:00:00Z")
 * в удобный для отображения вид "DD.MM.YYYY".
 * @param {string} iso - Дата из ProjectInfo.
 * @returns {string}
 */
function formatDate(iso) {
  return dayjs(iso).format("DD.MM.YYYY");
}

/**
 * Русские формы множественного числа для дней ("1 день", "2 дня", "5 дней").
 * @param {number} n - Количество дней (duration_days).
 * @returns {string}
 */
function pluralDays(n) {
  const mod10 = n % 10;
  const mod100 = n % 100;
  if (mod100 >= 11 && mod100 <= 14) return `${n} дней`;
  if (mod10 === 1) return `${n} день`;
  if (mod10 >= 2 && mod10 <= 4) return `${n} дня`;
  return `${n} дней`;
}

/**
 * Вкладка «Проект»: просмотр информации о проекте и редактирование
 * через EditProjectDialog (модальное окно).
 *
 * Интеграция с Tauri-командами через `invoke()`:
 *   - `get_project_info(projectId: null)` — данные выбранного проекта;
 *   - редактирование — в EditProjectDialog (`edit_project`).
 *
 * @param {Object} props
 * @param {number} props.dataVersion - Счётчик изменений данных приложения.
 */
export default function ProjectView({ dataVersion }) {
  /** @type {ProjectInfo|null} Данные текущего проекта. */
  const [project, setProject] = useState(null);
  /** @type {string|null} Ошибка загрузки (raw-сообщение от бэкенда). */
  const [error, setError] = useState(null);
  const [loading, setLoading] = useState(false);
  const [editOpen, setEditOpen] = useState(false);

  useEffect(() => {
    void loadProject();
    // loadProject стабильна — запускаем на монтировании и после изменений данных.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dataVersion]);

  /**
   * Запрашивает информацию о выбранном проекте через Tauri-команду.
   * При монтировании вьюхи (переключении на вкладку) всегда свежие данные.
   */
  async function loadProject() {
    setLoading(true);
    try {
      const info = await invoke("get_project_info", { projectId: null });
      setProject(info);
      setError(null);
    } catch (err) {
      setProject(null);
      setError(typeof err === "string" ? err : "Неизвестная ошибка");
    } finally {
      setLoading(false);
    }
  }

  /** Пустое состояние: проекта нет или загрузка не удалась. */
  if (!project) {
    const noProject = !error || NO_PROJECT_ERRORS.includes(error);
    return (
      <section className="view">
        <Typography.Title level={3}>Проект</Typography.Title>
        <Card>
          {loading ? (
            <div className="views-loading">
              <Spin />
            </div>
          ) : (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description={
                noProject
                  ? "Проект не создан. Создайте или откройте проект через меню «Файл»."
                  : "Не удалось загрузить данные проекта."
              }
            >
              {!noProject && (
                <Typography.Paragraph type="secondary">
                  {error}
                </Typography.Paragraph>
              )}
            </Empty>
          )}
        </Card>
      </section>
    );
  }

  return (
    <section className="view">
      <Typography.Title level={3}>Проект</Typography.Title>
      <Card>
        <Descriptions bordered size="medium">
          <Descriptions.Item label="Название">{project.name}</Descriptions.Item>
          <Descriptions.Item label="Описание">
            {project.description || "—"}
          </Descriptions.Item>
          <Descriptions.Item label="Дата начала">
            {formatDate(project.date_start)}
          </Descriptions.Item>
          <Descriptions.Item label="Дата окончания">
            {formatDate(project.date_end)}
          </Descriptions.Item>
          <Descriptions.Item label="Длительность">
            {pluralDays(project.duration_days)}
          </Descriptions.Item>
        </Descriptions>
        <Button
          type="primary"
          icon={<EditOutlined />}
          onClick={() => setEditOpen(true)}
        >
          Редактировать
        </Button>
      </Card>

      <EditProjectDialog
        open={editOpen}
        project={project}
        onClose={() => setEditOpen(false)}
        onSaved={loadProject}
      />
    </section>
  );
}
