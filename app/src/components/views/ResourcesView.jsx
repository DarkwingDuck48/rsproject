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
import { EXCEPTION_TYPE_LABELS, RATE_MEASURE_LABELS } from "../../lib/options";
import EditResourceDialog from "../dialogs/EditResourceDialog";
import NewResourceDialog from "../dialogs/NewResourceDialog";
import UnavailablePeriodDialog from "../dialogs/UnavailablePeriodDialog";

/** Форматирует дату RFC 3339 в DD.MM.YYYY. */
function formatDate(iso) {
  if (!iso) return "—";
  return iso.slice(0, 10).split("-").reverse().join(".");
}

/**
 * Вкладка «Ресурсы»: таблица ресурсов с периодами недоступности
 * и действиями Add / Edit / Delete / Add Unavailable Period.
 *
 * @param {Object} props
 * @param {number} props.dataVersion - Счётчик изменений данных приложения.
 */
export default function ResourcesView({ dataVersion }) {
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
    } catch {
      setResources([]);
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

  async function handleDelete() {
    if (!selectedResource) return;
    try {
      await deleteResource(selectedResource.id);
      message.success("Ресурс удалён");
      setSelectedId(null);
      await load();
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
                key={`${period.period.date_start}-${period.period.date_end}`}
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

      <NewResourceDialog
        open={newOpen}
        onClose={() => setNewOpen(false)}
        onCreated={load}
      />
      <EditResourceDialog
        open={editingResource != null}
        resource={editingResource}
        onClose={() => setEditingResource(null)}
        onSaved={load}
      />
      <UnavailablePeriodDialog
        open={periodResource != null}
        resource={periodResource}
        onClose={() => setPeriodResource(null)}
        onSaved={load}
      />
    </section>
  );
}
