import { Button, Form, Modal, Space, message } from "antd";
import { SaveOutlined } from "@ant-design/icons";
import dayjs from "dayjs";
import { useState } from "react";
import { editProject } from "../../lib/api";
import ProjectFormFields, { PROJECT_DATE_FORMAT } from "./ProjectFormFields";

/**
 * Диалог редактирования текущего проекта.
 *
 * @param {Object}    props
 * @param {boolean}   props.open        - Открыт ли диалог.
 * @param {ProjectInfo} props.project   - Текущие данные проекта (для предзаполнения).
 * @param {Function}  props.onClose     - Закрыть диалог: () => void.
 * @param {Function}  props.onSaved     - Изменения сохранены: () => void.
 */
export default function EditProjectDialog({ open, project, onClose, onSaved }) {
  const [saving, setSaving] = useState(false);

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
      await editProject(
        values.name,
        values.description,
        start.format(PROJECT_DATE_FORMAT),
        end.format(PROJECT_DATE_FORMAT),
      );
      message.success("Изменения сохранены");
      onClose();
      onSaved();
    } catch (err) {
      message.error(`Не удалось сохранить изменения: ${err}`);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal
      open={open}
      title="Редактирование проекта"
      onCancel={onClose}
      footer={null}
      destroyOnHidden
    >
      <Form
        layout="vertical"
        initialValues={{
          name: project?.name,
          description: project?.description,
          date_start: dayjs(project?.date_start),
          date_end: dayjs(project?.date_end),
        }}
        onFinish={handleFinish}
      >
        <ProjectFormFields />
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
    </Modal>
  );
}
