import { Button, DatePicker, Form, InputNumber, Modal, Select, Space, Typography, message } from "antd";
import { TeamOutlined } from "@ant-design/icons";
import dayjs from "dayjs";
import { useEffect, useState } from "react";
import { assignResource, getResources } from "../../lib/api";

/** Формат дат для бэкенда. */
const DATE_FORMAT = "YYYY-MM-DD";

/**
 * Диалог назначения ресурса на задачу.
 *
 * Допустимо либо указать обе даты окна (в пределах задачи), либо ни одной —
 * тогда ресурс назначается на всю задачу.
 *
 * @param {Object}    props
 * @param {boolean}   props.open       - Открыт ли диалог.
 * @param {TaskInfo}  props.task       - Задача, на которую назначается ресурс.
 * @param {Function}  props.onClose    - Закрыть диалог: () => void.
 * @param {Function}  props.onSaved    - Назначение выполнено: () => void.
 */
export default function AssignResourceDialog({ open, task, onClose, onSaved }) {
  const [resources, setResources] = useState([]);
  const [saving, setSaving] = useState(false);

  // При открытии подгружаем список ресурсов
  useEffect(() => {
    if (open) {
      getResources()
        .then(setResources)
        .catch(() => setResources([]));
    }
  }, [open]);

  async function handleFinish(values) {
    const dateStart = values.date_start ? dayjs(values.date_start).format(DATE_FORMAT) : null;
    const dateEnd = values.date_end ? dayjs(values.date_end).format(DATE_FORMAT) : null;

    if ((dateStart === null) !== (dateEnd === null)) {
      message.error("Укажите обе даты окна либо ни одной");
      return;
    }
    if (dateStart && dateEnd && dayjs(dateStart).isAfter(dayjs(dateEnd))) {
      message.error("Дата окончания окна не может быть раньше даты начала");
      return;
    }

    setSaving(true);
    try {
      await assignResource(task.id, values.resource_id, values.engagement, dateStart, dateEnd);
      message.success("Ресурс назначен");
      onClose();
      onSaved();
    } catch (err) {
      message.error(`Не удалось назначить ресурс: ${err}`);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal open={open} title={`Назначить ресурс: ${task?.name ?? ""}`} footer={null} destroyOnHidden>
      <Form layout="vertical" initialValues={{ engagement: 1 }} onFinish={handleFinish}>
        <Form.Item
          name="resource_id"
          label="Ресурс"
          rules={[{ required: true, message: "Выберите ресурс" }]}
        >
          <Select
            placeholder="Ресурс"
            options={resources.map((resource) => ({
              value: resource.id,
              label: resource.name,
            }))}
            style={{ width: "100%" }}
          />
        </Form.Item>
        <Form.Item
          name="engagement"
          label="Занятость (доля 0–1)"
          rules={[{ required: true, message: "Укажите долю занятости" }]}
        >
          <InputNumber
            min={0.01}
            max={1}
            step={0.1}
            precision={2}
            style={{ width: "100%" }}
          />
        </Form.Item>
        <Typography.Paragraph type="secondary">
          Временное окно назначения (необязательно): если не указано,
          ресурс назначается на всю длительность задачи.
        </Typography.Paragraph>
        <Form.Item name="date_start" label="Начало окна">
          <DatePicker format={DATE_FORMAT} style={{ width: "100%" }} />
        </Form.Item>
        <Form.Item name="date_end" label="Окончание окна">
          <DatePicker format={DATE_FORMAT} style={{ width: "100%" }} />
        </Form.Item>
        <Form.Item>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              icon={<TeamOutlined />}
              loading={saving}
            >
              Назначить
            </Button>
            <Button onClick={onClose}>Отмена</Button>
          </Space>
        </Form.Item>
      </Form>
    </Modal>
  );
}
