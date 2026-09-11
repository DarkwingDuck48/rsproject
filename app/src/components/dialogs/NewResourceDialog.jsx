import { Button, Form, Input, InputNumber, Modal, Select, Space, message } from "antd";
import { FileAddOutlined } from "@ant-design/icons";
import { useState } from "react";
import { addResource } from "../../lib/api";
import { RATE_MEASURE_OPTIONS } from "../../lib/options";

/**
 * Диалог создания ресурса.
 *
 * @param {Object}   props
 * @param {boolean}  props.open       - Открыт ли диалог.
 * @param {Function} props.onClose    - Закрыть диалог: () => void.
 * @param {Function} props.onCreated  - Ресурс создан: () => void.
 */
export default function NewResourceDialog({ open, onClose, onCreated }) {
  const [saving, setSaving] = useState(false);

  async function handleFinish(values) {
    setSaving(true);
    try {
      await addResource(values.name, values.rate, values.rate_measure);
      message.success("Ресурс создан");
      onClose();
      onCreated();
    } catch (err) {
      message.error(`Не удалось создать ресурс: ${err}`);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal open={open} title="Новый ресурс" footer={null} destroyOnHidden>
      <Form layout="vertical" onFinish={handleFinish}>
        <Form.Item
          name="name"
          label="Название"
          rules={[{ required: true, whitespace: true, message: "Введите название" }]}
        >
          <Input placeholder="Название ресурса" />
        </Form.Item>
        <Form.Item
          name="rate"
          label="Ставка"
          rules={[{ required: true, message: "Укажите ставку" }]}
        >
          <InputNumber
            min={0}
            precision={2}
            placeholder="Ставка"
            style={{ width: "100%" }}
          />
        </Form.Item>
        <Form.Item
          name="rate_measure"
          label="Мера ставки"
          rules={[{ required: true, message: "Выберите меру ставки" }]}
        >
          <Select options={RATE_MEASURE_OPTIONS} style={{ width: "100%" }} />
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
