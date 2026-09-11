import {
  Button,
  Card,
  Empty,
  Space,
  Spin,
  Table,
  Tag,
  Typography,
  message,
} from "antd";
import {
  CalendarOutlined,
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
} from "@ant-design/icons";
import { useEffect, useState } from "react";
import { deleteResource, getResources } from "../../lib/api";
import { isNoProjectError } from "../../lib/errors";
import { formatDate } from "../../lib/format";
import { useHotkey } from "../../hooks/useHotkey";
import { EXCEPTION_TYPE_LABELS, RATE_MEASURE_LABELS } from "../../lib/options";
import EditResourceDialog from "../dialogs/EditResourceDialog";
import NewResourceDialog from "../dialogs/NewResourceDialog";
import UnavailablePeriodDialog from "../dialogs/UnavailablePeriodDialog";

/**
 * Вкладка «Ресурсы»: таблица ресурсов с периодами недоступности
 * и действиями Add / Edit / Delete / Add Unavailable Period.
 *
 * @param {Object} props
 * @param {number} props.dataVersion - Счётчик изменений данных приложения.
 * @param {Function} props.onDataChange - Уведомить приложение об изменении данных (5.18).
 */
export default function ResourcesView({ dataVersion, onDataChange }) {
  const [resources, setResources] = useState([]);
  const [loading, setLoading] = useState(false);
  const [selectedId, setSelectedId] = useState(null);
  // Управление диалогами
  const [newOpen, setNewOpen] = useState(false);
  const [editingResource, setEditingResource] = useState(null);
  const [periodResource, setPeriodResource] = useState(null);

  async function load() {
    setLoading(true);
    try {
      setResources(await getResources());
    } catch (err) {
      setResources([]);
      if (!isNoProjectError(err)) {
        message.error(`Не удалось загрузить ресурсы: ${err}`);
      }
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void load();
  }, [dataVersion]);

  const selectedResource = resources.find(
    (resource) => resource.id === selectedId,
  );

  // Delete удаляет выбранный ресурс (задача 5.17). Пока открыт любой диалог —
  // не удаляем ничего «под модалкой».
  const dialogOpen =
    newOpen || editingResource !== null || periodResource !== null;
  useHotkey(
    "Delete",
    () => {
      if (!dialogOpen) void handleDelete();
    },
    { ignoreTyping: true },
  );

  async function handleDelete() {
    if (!selectedResource) return;
    try {
      await deleteResource(selectedResource.id);
      message.success("Ресурс удалён");
      setSelectedId(null);
      // Глобальное изменение — пусть статус-бар и другие панели перечитают (5.18).
      onDataChange();
    } catch (err) {
      message.error(`Не удалось удалить ресурс: ${err}`);
    }
  }

  const columns = [
    {
      title: "Название",
      dataIndex: "name",
    },
    {
      title: "Ставка",
      dataIndex: "rate",
      align: "right",
      render: (value) => value.toFixed(2),
    },
    {
      title: "Мера",
      dataIndex: "rate_measure",
      render: (value) => RATE_MEASURE_LABELS[value] ?? value,
    },
    {
      title: "Периоды недоступности",
      dataIndex: "unavailable_periods",
      render: (value) =>
        value.length === 0 ? (
          <Typography.Text type="secondary">—</Typography.Text>
        ) : (
          <Space wrap size={4}>
            {value.map((period) => (
              <Tag
                key={`${period.period.date_start}-${period.period.date_end}-${period.exception_type}`}
              >
                {EXCEPTION_TYPE_LABELS[period.exception_type] ??
                  period.exception_type}
                : {formatDate(period.period.date_start)}–
                {formatDate(period.period.date_end)}
              </Tag>
            ))}
          </Space>
        ),
    },
  ];

  return (
    <section className="view">
      <Typography.Title level={3}>Ресурсы</Typography.Title>

      <Space className="views-toolbar">
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => setNewOpen(true)}
        >
          Добавить
        </Button>
        <Button
          icon={<EditOutlined />}
          disabled={!selectedResource}
          onClick={() => setEditingResource(selectedResource)}
        >
          Редактировать
        </Button>
        <Button
          danger
          icon={<DeleteOutlined />}
          disabled={!selectedResource}
          onClick={handleDelete}
        >
          Удалить
        </Button>
        <Button
          icon={<CalendarOutlined />}
          disabled={!selectedResource}
          onClick={() => setPeriodResource(selectedResource)}
        >
          Период недоступности
        </Button>
      </Space>

      {loading ? (
        <div className="views-loading">
          <Spin />
        </div>
      ) : resources.length === 0 ? (
        <Card>
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description="Ресурсов нет. Добавьте первый ресурс."
          />
        </Card>
      ) : (
        <Table
          dataSource={resources}
          rowKey="id"
          columns={columns}
          rowSelection={{
            type: "radio",
            selectedRowKeys: selectedId ? [selectedId] : [],
            onChange: (keys) => setSelectedId(keys.length ? keys[0] : null),
          }}
          pagination={false}
        />
      )}

      {/* Создание/редактирование/период меняют данные глобально —
          инкремент dataVersion перезагрузит вьюху и статус-бар (5.18). */}
      <NewResourceDialog
        open={newOpen}
        onClose={() => setNewOpen(false)}
        onCreated={onDataChange}
      />
      <EditResourceDialog
        open={editingResource != null}
        resource={editingResource}
        onClose={() => setEditingResource(null)}
        onSaved={onDataChange}
      />
      <UnavailablePeriodDialog
        open={periodResource != null}
        resource={periodResource}
        onClose={() => setPeriodResource(null)}
        onSaved={onDataChange}
      />
    </section>
  );
}
