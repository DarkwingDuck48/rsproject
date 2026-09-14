import { Button, DatePicker, Form, Modal, Select, Space, message } from "antd";
import { CalendarOutlined } from "@ant-design/icons";
import dayjs from "dayjs";
import { useState } from "react";
import { addUnavailablePeriod } from "../../lib/api";
import { EXCEPTION_TYPE_OPTIONS } from "../../lib/options";

/** Формат дат для бэкенда. */
const DATE_FORMAT = "YYYY-MM-DD";

/**
 * Диалог добавления периода недоступности ресурса.
 *
 * @param {Object}   props
 * @param {boolean}  props.open       - Открыт ли диалог.
 * @param {ResourceInfo} props.resource - Ресурс, для которого добавляется период.
 * @param {Function} props.onClose    - Закрыть диалог: () => void.
 * @param {Function} props.onSaved    - Период добавлен: () => void.
 */
export default function UnavailablePeriodDialog({
  open,
  resource,
  onClose,
  onSaved,
}) {
  const [saving, setSaving] = useState(false);

  async function handleFinish(values) {
    const start = dayjs(values.date_start);
    const end = dayjs(values.date_end);

    if (!start.isValid() || !end.isValid()) {
      message.error("Укажите даты начала и окончания периода");
      return;
    }
    // TimeWindow::new требует start < end — период минимум 1 день
    if (!start.isBefore(end)) {
      message.error(
        "Дата окончания должна быть позже даты начала (период минимум 1 день)",
      );
      return;
    }

    setSaving(true);
    try {
      await addUnavailablePeriod(
        resource.id,
        start.format(DATE_FORMAT),
        end.format(DATE_FORMAT),
        values.exception_type,
      );
      message.success("Период недоступности добавлен");
      onClose();
      onSaved();
    } catch (err) {
      message.error(`Не удалось добавить период: ${err}`);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal
      open={open}
      title={`Период недоступности: ${resource?.name ?? ""}`}
      onCancel={onClose}
      footer={null}
      destroyOnHidden
    >
      <Form layout="vertical" onFinish={handleFinish}>
        <Form.Item
          name="exception_type"
          label="Причина"
          rules={[{ required: true, message: "Выберите причину" }]}
        >
          <Select options={EXCEPTION_TYPE_OPTIONS} style={{ width: "100%" }} />
        </Form.Item>
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
        <Form.Item>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              icon={<CalendarOutlined />}
              loading={saving}
            >
              Добавить
            </Button>
            <Button onClick={onClose}>Отмена</Button>
          </Space>
        </Form.Item>
      </Form>
    </Modal>
  );
}
