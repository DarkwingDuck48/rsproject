import { Button, Form, Modal, Space, message } from "antd";
import { FileAddOutlined } from "@ant-design/icons";
import dayjs from "dayjs";
import { useState } from "react";
import { createProject } from "../../lib/api";
import ProjectFormFields from "./ProjectFormFields";
import { PROJECT_DATE_FORMAT } from "./ProjectFormFields";

/**
 * Диалог создания нового проекта.
 *
 * @param {Object}   props
 * @param {boolean}  props.open      - Открыт ли диалог.
 * @param {Function} props.onClose   - Закрыть диалог: () => void.
 * @param {Function} props.onCreated - Проект создан: () => void.
 */
export default function NewProjectDialog({ open, onClose, onCreated }) {
  const [saving, setSaving] = useState(false);

  /**
   * Сохраняет проект: валидирует порядок дат и вызывает команду `create_project`.
   * @param {Object} values - Значения полей формы (даты — dayjs-объекты).
   */
  async function handleFinish(values) {
    const start = dayjs(values.date_start);
    const end = dayjs(values.date_end);

    if (!start.isValid() || !end.isValid()) {
      message.error("Укажите даты начала и окончания проекта");
      return;
    }
    if (start.isAfter(end)) {
      message.error("Дата окончания не может быть раньше даты начала");
      return;
    }

    setSaving(true);
    try {
      await createProject(
        values.name,
        values.description,
        start.format(PROJECT_DATE_FORMAT),
        end.format(PROJECT_DATE_FORMAT),
      );
      message.success("Проект создан");
      onClose();
      onCreated();
    } catch (err) {
      message.error(`Не удалось создать проект: ${err}`);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal
      open={open}
      title="Новый проект"
      onCancel={onClose}
      footer={null}
      destroyOnHidden
    >
      <Form layout="vertical" onFinish={handleFinish}>
        <ProjectFormFields />
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
