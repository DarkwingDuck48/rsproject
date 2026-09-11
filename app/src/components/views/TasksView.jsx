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
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  TeamOutlined,
} from "@ant-design/icons";
import { useEffect, useState } from "react";
import {
  deleteTask,
  getTaskTree,
  recalculateCriticalPath,
} from "../../lib/api";
import { TASK_STATUS_LABELS } from "../../lib/options";
import AssignResourceDialog from "../dialogs/AssignResourceDialog";
import NewTaskDialog from "../dialogs/NewTaskDialog";
import TaskDetailsDialog from "../dialogs/TaskDetailsDialog";

/** Форматирует дату RFC 3339 в DD.MM.YYYY. */
function formatDate(iso) {
  if (!iso) return "—";
  return iso.slice(0, 10).split("-").reverse().join(".");
}

/**
 * Преобразует дерево задач (TaskTreeNode[]) в записи таблицы:
 * каждый узел — TaskInfo + вложенный `children` (для tree-режима Table).
 * @param {TaskTreeNode[]} nodes
 * @returns {Array<Object>}
 */
function toRows(nodes) {
  return nodes.map((node) => ({
    ...node.task,
    children: toRows(node.children),
  }));
}

/**
 * Вкладка «Задачи»: дерево задач с иерархией и действиями
 * Add / Edit / Delete / Assign Resource.
 *
 * @param {Object} props
 * @param {number} props.dataVersion - Счётчик изменений данных приложения (для перезагрузки).
 */
export default function TasksView({ dataVersion }) {
  const [rows, setRows] = useState([]);
  const [tree, setTree] = useState([]);
  const [loading, setLoading] = useState(false);
  const [selectedId, setSelectedId] = useState(null);
  // Управление диалогами
  const [newOpen, setNewOpen] = useState(false);
  const [detailsTaskId, setDetailsTaskId] = useState(null);
  const [assignTaskId, setAssignTaskId] = useState(null);

  async function load() {
    setLoading(true);
    try {
      const taskTree = await getTaskTree();
      setTree(taskTree);
      setRows(toRows(taskTree));
    } catch {
      setTree([]);
      setRows([]);
    } finally {
      setLoading(false);
    }
  }

  // Загрузка при монтировании вкладки и после изменения данных
  useEffect(() => {
    void load();
  }, [dataVersion]);

  const selectedTask = rows.find((row) => row.id === selectedId);
  // Деривируем объекты из свежих данных: после `load()` диалоги
  // получают обновлённый список зависимостей/назначений.
  const detailsTask = rows.find((row) => row.id === detailsTaskId);
  const assignTask = rows.find((row) => row.id === assignTaskId);

  async function handleDelete() {
    if (!selectedTask) return;
    try {
      await deleteTask(selectedTask.id);
      message.success("Задача удалена");
      setSelectedId(null);
      await load();
    } catch (err) {
      message.error(`Не удалось удалить задачу: ${err}`);
    }
  }

  async function handleRecalc() {
    try {
      const path = await recalculateCriticalPath();
      message.success(`Критический путь: ${path.length} задач`);
    } catch (err) {
      message.error(`Не удалось рассчитать критический путь: ${err}`);
    }
  }

  const columns = [
    {
      title: "Название",
      dataIndex: "name",
      render: (value, record) => (
        <Space size={4}>
          {record.is_summary && <Tag>Сводная</Tag>}
          <span>{value}</span>
        </Space>
      ),
    },
    {
      title: "Начало",
      dataIndex: "date_start",
      render: (value) => formatDate(value),
    },
    {
      title: "Окончание",
      dataIndex: "date_end",
      render: (value) => formatDate(value),
    },
    {
      title: "Длит., дн.",
      dataIndex: "duration_days",
      align: "right",
    },
    {
      title: "Статус",
      dataIndex: "status",
      render: (value) => TASK_STATUS_LABELS[value] ?? value,
    },
    {
      title: "Стоимость",
      dataIndex: "cost",
      align: "right",
      render: (value) => (value ?? 0).toFixed(2),
    },
  ];

  return (
    <section className="view">
      <Typography.Title level={3}>Задачи</Typography.Title>

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
          disabled={!selectedTask}
          onClick={() => setDetailsTaskId(selectedTask.id)}
        >
          Редактировать
        </Button>
        <Button
          danger
          icon={<DeleteOutlined />}
          disabled={!selectedTask}
          onClick={handleDelete}
        >
          Удалить
        </Button>
        <Button
          icon={<TeamOutlined />}
          disabled={!selectedTask}
          onClick={() => setAssignTaskId(selectedTask.id)}
        >
          Назначить ресурс
        </Button>
        <Button type="dashed" onClick={handleRecalc}>
          Рассчитать критический путь
        </Button>
      </Space>

      {loading ? (
        <div className="views-loading">
          <Spin />
        </div>
      ) : rows.length === 0 ? (
        <Card>
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description="Задач нет. Создайте первую задачу."
          />
        </Card>
      ) : (
        <Table
          dataSource={rows}
          rowKey="id"
          columns={columns}
          expandable={{
            childrenColumnName: "children",
            defaultExpandAllRows: true,
          }}
          rowSelection={{
            type: "radio",
            selectedRowKeys: selectedId ? [selectedId] : [],
            onChange: (keys) => setSelectedId(keys.length ? keys[0] : null),
          }}
          pagination={false}
        />
      )}

      <NewTaskDialog
        open={newOpen}
        onClose={() => setNewOpen(false)}
        onCreated={load}
      />
      <TaskDetailsDialog
        open={detailsTask != null}
        task={detailsTask}
        taskTree={tree}
        onClose={() => setDetailsTaskId(null)}
        onSaved={load}
      />
      <AssignResourceDialog
        open={assignTask != null}
        task={assignTask}
        onClose={() => setAssignTaskId(null)}
        onSaved={load}
      />
    </section>
  );
}
