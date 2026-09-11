import {
  Button,
  DatePicker,
  Divider,
  Form,
  Input,
  InputNumber,
  Modal,
  Select,
  Space,
  Tag,
  TreeSelect,
  Typography,
  message,
} from "antd";
import { DeleteOutlined, PlusOutlined, SaveOutlined } from "@ant-design/icons";
import dayjs from "dayjs";
import { useEffect, useState } from "react";
import { addDependency, editTask, removeDependency } from "../../lib/api";
import {
  DEPENDENCY_TYPE_LABELS,
  DEPENDENCY_TYPE_OPTIONS,
  TASK_STATUS_OPTIONS,
} from "../../lib/options";

/** Формат дат для бэкенда (%Y-%m-%d). */
const DATE_FORMAT = "YYYY-MM-DD";

/**
 * Строит treeData для TreeSelect из дерева задач, исключая заданные ID
 * (саму задачу — нельзя сделать задачу потомком самой себя — и уже
 * добавленные предшественники).
 *
 * @param {TaskTreeNode[]} nodes
 * @param {string[]} excludeIds - UUID'ы задач, которые нельзя выбирать
 * @returns {{value: string, label: string, children: Object[]}[]}
 */
function toTreeData(nodes, excludeIds) {
  return nodes
    .filter((node) => !excludeIds.includes(node.task.id))
    .map((node) => ({
      value: node.task.id,
      label: node.task.name,
      children: toTreeData(node.children, excludeIds),
    }));
}

/**
 * Ищет имя задачи по ID в дереве.
 * @param {TaskTreeNode[]} nodes
 * @param {string} id
 * @returns {?string}
 */
function findTaskName(nodes, id) {
  for (const node of nodes) {
    if (node.task.id === id) return node.task.name;
    const found = findTaskName(node.children, id);
    if (found) return found;
  }
  return null;
}

/**
 * Диалог просмотра и редактирования задачи: параметры + управление зависимостями.
 *
 * @param {Object}   props
 * @param {boolean}  props.open     - Открыт ли диалог.
 * @param {TaskInfo} props.task     - Редактируемая задача.
 * @param {TaskTreeNode[]} props.taskTree - Дерево задач.
 * @param {Function} props.onClose  - Закрыть диалог: () => void.
 * @param {Function} props.onSaved  - Данные изменены (сохранение/зависимость): () => void.
 */
export default function TaskDetailsDialog({
  open,
  task,
  taskTree,
  onClose,
  onSaved,
}) {
  const [saving, setSaving] = useState(false);
  const [depOn, setDepOn] = useState(null);
  const [depType, setDepType] = useState("Blocking");
  const [depLag, setDepLag] = useState(null);

  const dependencies = task?.dependencies ?? [];
  // Задачи, недоступные для выбора родителем/предшественником
  const excludeIds = [
    task?.id,
    ...dependencies.map((dep) => dep.depends_on),
  ].filter(Boolean);

  // При открытии сбрасываем поля «добавить зависимость»
  useEffect(() => {
    if (open) {
      setDepOn(null);
      setDepType("Blocking");
      setDepLag(null);
    }
  }, [open]);

  async function handleFinish(values) {
    // У сводных задач дат нет — проверяем только у обычных
    if (!task.is_summary) {
      const start = dayjs(values.date_start);
      const end = dayjs(values.date_end);
      if (!start.isValid() || !end.isValid()) {
        message.error("Укажите даты начала и окончания задачи");
        return;
      }
      // Бэкенд (Task::new_regular) требует start < end — задача минимум 1 день
      if (!start.isBefore(end)) {
        message.error(
          "Дата окончания должна быть позже даты начала (длительность минимум 1 день)",
        );
        return;
      }
    }

    setSaving(true);
    try {
      await editTask(task.id, {
        name: values.name,
        // Сводные задачи даты не имеют — бэкенд отклоняет их установку
        dateStart: task.is_summary
          ? null
          : dayjs(values.date_start).format(DATE_FORMAT),
        dateEnd: task.is_summary
          ? null
          : dayjs(values.date_end).format(DATE_FORMAT),
        status: values.status,
        parentId: values.parent_id || null,
      });
      message.success("Задача обновлена");
      onClose();
      onSaved();
    } catch (err) {
      message.error(`Не удалось обновить задачу: ${err}`);
    } finally {
      setSaving(false);
    }
  }

  async function handleAddDependency() {
    if (!depOn) {
      message.error("Выберите задачу-предшественника");
      return;
    }
    try {
      await addDependency(task.id, depOn, depType, depLag);
      message.success("Зависимость добавлена");
      setDepOn(null);
      setDepLag(null);
      onSaved();
    } catch (err) {
      message.error(`Не удалось добавить зависимость: ${err}`);
    }
  }

  async function handleRemoveDependency(dependsOn) {
    try {
      await removeDependency(task.id, dependsOn);
      message.success("Зависимость удалена");
      onSaved();
    } catch (err) {
      message.error(`Не удалось удалить зависимость: ${err}`);
    }
  }

  return (
    <Modal
      open={open}
      title={task ? `Задача: ${task.name}` : "Задача"}
      footer={null}
    >
      <Form
        layout="vertical"
        initialValues={{
          name: task?.name,
          date_start: dayjs(task?.date_start),
          date_end: dayjs(task?.date_end),
          status: task?.status,
          parent_id: task?.parent_id,
        }}
        onFinish={handleFinish}
      >
        <Form.Item
          name="name"
          label="Название"
          rules={[
            {
              required: true,
              whitespace: true,
              message: "Введите название задачи",
            },
          ]}
        >
          <Input placeholder="Название задачи" />
        </Form.Item>
        <Form.Item name="status" label="Статус">
          <Select options={TASK_STATUS_OPTIONS} style={{ width: "100%" }} />
        </Form.Item>
        {!task?.is_summary && (
          <>
            <Form.Item
              name="date_start"
              label="Дата начала"
              rules={[{ required: true, message: "Укажите дату начала" }]}
            >
              <DatePicker format={DATE_FORMAT} style={{ width: "100%" }} />
            </Form.Item>
            <Form.Item
              name="date_end"
              label="Дата окончания"
              rules={[{ required: true, message: "Укажите дату окончания" }]}
            >
              <DatePicker format={DATE_FORMAT} style={{ width: "100%" }} />
            </Form.Item>
          </>
        )}
        {task?.is_summary && (
          <Typography.Paragraph type="secondary">
            У сводной задачи даты вычисляются автоматически по подзадачам.
          </Typography.Paragraph>
        )}
        <Form.Item name="parent_id" label="Родительская задача (группа)">
          <TreeSelect
            allowClear
            placeholder="Корневая (без родителя)"
            treeData={toTreeData(taskTree, [task?.id])}
            treeNodeLabelProp="label"
            fieldNames={{
              value: "value",
              label: "label",
              children: "children",
            }}
            style={{ width: "100%" }}
          />
        </Form.Item>
        <Form.Item>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              icon={<SaveOutlined />}
              loading={saving}
            >
              Сохранить
            </Button>
            <Button onClick={onClose}>Отмена</Button>
          </Space>
        </Form.Item>
      </Form>

      <Divider />

      <Typography.Title level={5}>
        Зависимости (предшественники)
      </Typography.Title>
      <Space direction="vertical" style={{ width: "100%" }} size="small">
        {dependencies.length === 0 ? (
          <Typography.Paragraph type="secondary" style={{ marginBottom: 0 }}>
            Зависимостей нет.
          </Typography.Paragraph>
        ) : (
          dependencies.map((dep) => (
            <div key={dep.depends_on} className="task-dependency-row">
              <Tag>{DEPENDENCY_TYPE_LABELS[dep.dependency_type]}</Tag>
              <span className="task-dependency-row__name">
                {findTaskName(taskTree, dep.depends_on) ?? dep.depends_on}
              </span>
              {dep.lag_days != null && (
                <Tag color="blue">лаг {dep.lag_days} дн.</Tag>
              )}
              <Button
                type="text"
                danger
                size="small"
                icon={<DeleteOutlined />}
                onClick={() => handleRemoveDependency(dep.depends_on)}
              />
            </div>
          ))
        )}

        <Divider style={{ margin: "8px 0" }} />

        <Typography.Text type="secondary">
          Добавить зависимость:
        </Typography.Text>
        <Space direction="vertical" size="small" style={{ width: "100%" }}>
          <TreeSelect
            placeholder="Задача-предшественник"
            treeData={toTreeData(taskTree, excludeIds)}
            treeNodeLabelProp="label"
            fieldNames={{
              value: "value",
              label: "label",
              children: "children",
            }}
            value={depOn}
            onChange={setDepOn}
            style={{ width: "100%" }}
          />
          <Space>
            <Select
              options={DEPENDENCY_TYPE_OPTIONS}
              value={depType}
              onChange={setDepType}
              style={{ width: 180 }}
            />
            <InputNumber
              placeholder="Лаг (дней)"
              min={0}
              value={depLag}
              onChange={setDepLag}
              style={{ width: 140 }}
            />
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={handleAddDependency}
            >
              Добавить
            </Button>
          </Space>
        </Space>
      </Space>
    </Modal>
  );
}
