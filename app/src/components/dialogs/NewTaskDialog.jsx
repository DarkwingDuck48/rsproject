import {
  Button,
  DatePicker,
  Form,
  Input,
  Modal,
  Space,
  Switch,
  TreeSelect,
  message,
} from "antd";
import { FileAddOutlined } from "@ant-design/icons";
import dayjs from "dayjs";
import { useEffect, useState } from "react";
import { addTask, getTaskTree } from "../../lib/api";
import { isNoProjectError } from "../../lib/errors";

/** Формат дат для бэкенда (`%Y-%m-%d`). */
const DATE_FORMAT = "YYYY-MM-DD";

/**
 * Строит treeData для TreeSelect из дерева задач.
 * Исключает задачу с его id (нельзя сделать задачу потомком самой себя или её потомков).
 * @param {TaskTreeNode[]} nodes
 * @param {string} excludeId - UUID редактируемой задачи
 * @returns {{value: string, label: string, children: Object[]}[]}
 */
function toTreeData(nodes, excludeId = null) {
  return nodes
    .filter((node) => node.task.id !== excludeId)
    .map((node) => ({
      value: node.task.id,
      label: node.task.name,
      children: toTreeData(node.children, excludeId),
    }));
}

/**
 * Диалог создания задачи (обычной или сводной).
 *
 * @param {Object}   props
 * @param {boolean}  props.open       - Открыт ли диалог.
 * @param {Function} props.onClose    - Закрыть диалог: () => void.
 * @param {Function} props.onCreated  - Задача создана: () => void.
 */
export default function NewTaskDialog({ open, onClose, onCreated }) {
  const [taskTree, setTaskTree] = useState([]);
  const [isSummary, setIsSummary] = useState(false);
  const [saving, setSaving] = useState(false);

  // При каждом открытии подгружаем дерево задач (для выбора родителя)
  // и сбрасываем переключатель «сводная задача».
  useEffect(() => {
    if (open) {
      setIsSummary(false);
      getTaskTree()
        .then(setTaskTree)
        .catch((err) => {
          setTaskTree([]);
          if (!isNoProjectError(err)) {
            message.error(`Не удалось загрузить задачи: ${err}`);
          }
        });
    }
  }, [open]);

  async function handleFinish(values) {
    const parentId = values.parent_id || null;
    if (isSummary) {
      setSaving(true);
      try {
        await addTask(values.name, null, null, true, parentId);
        message.success("Сводная задача создана");
        onClose();
        onCreated();
      } catch (err) {
        message.error(`Не удалось создать задачу: ${err}`);
      } finally {
        setSaving(false);
      }
      return;
    }

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

    setSaving(true);
    try {
      await addTask(
        values.name,
        start.format(DATE_FORMAT),
        end.format(DATE_FORMAT),
        false,
        parentId,
      );
      message.success("Задача создана");
      onClose();
      onCreated();
    } catch (err) {
      message.error(`Не удалось создать задачу: ${err}`);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal
      open={open}
      title="Новая задача"
      onCancel={onClose}
      footer={null}
      destroyOnHidden
    >
      <Form layout="vertical" onFinish={handleFinish}>
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
        <Form.Item name="is_summary" label="Сводная задача (группа)">
          <Switch checked={isSummary} onChange={setIsSummary} />
        </Form.Item>
        {!isSummary && (
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
        <Form.Item name="parent_id" label="Родительская задача (группа)">
          <TreeSelect
            allowClear
            placeholder="Корневая (без родителя)"
            treeData={toTreeData(taskTree)}
            treeNodeLabelProp="label"
            fieldNames={{
              value: "value",
              label: "label",
              children: "children",
            }}
            showSearch
            style={{ width: "100%" }}
          />
        </Form.Item>
        <Form.Item>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              icon={<FileAddOutlined />}
              loading={saving}
            >
              Создать
            </Button>
            <Button onClick={onClose}>Отмена</Button>
          </Space>
        </Form.Item>
      </Form>
    </Modal>
  );
}
