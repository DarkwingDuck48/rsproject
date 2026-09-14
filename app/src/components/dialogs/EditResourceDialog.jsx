import {
  Button,
  Form,
  Input,
  InputNumber,
  Modal,
  Select,
  Space,
  message,
} from "antd";
import { SaveOutlined } from "@ant-design/icons";
import { useState } from "react";
import { editResource } from "../../lib/api";
import { RATE_MEASURE_OPTIONS } from "../../lib/options";

/**
 * Диалог редактирования ресурса.
 *
 * @param {Object}     props
 * @param {boolean}    props.open       - Открыт ли диалог.
 * @param {ResourceInfo} props.resource - Редактируемый ресурс.
 * @param {Function}   props.onClose    - Закрыть диалог: () => void.
 * @param {Function}   props.onSaved    - Изменения сохранены: () => void.
 */
export default function EditResourceDialog({
  open,
  resource,
  onClose,
  onSaved,
}) {
  const [saving, setSaving] = useState(false);

  async function handleFinish(values) {
    setSaving(true);
    try {
      await editResource(resource.id, {
        name: values.name,
        rate: values.rate,
        rateMeasure: values.rate_measure,
      });
      message.success("Ресурс обновлён");
      onClose();
      onSaved();
    } catch (err) {
      message.error(`Не удалось обновить ресурс: ${err}`);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal
      open={open}
      title={`Ресурс: ${resource?.name ?? ""}`}
      onCancel={onClose}
      footer={null}
      destroyOnHidden
    >
      <Form
        layout="vertical"
        initialValues={{
          name: resource?.name,
          rate: resource?.rate,
          rate_measure: resource?.rate_measure,
        }}
        onFinish={handleFinish}
      >
        <Form.Item
          name="name"
          label="Название"
          rules={[
            { required: true, whitespace: true, message: "Введите название" },
          ]}
        >
          <Input placeholder="Название ресурса" />
        </Form.Item>
        <Form.Item
          name="rate"
          label="Ставка"
          rules={[{ required: true, message: "Укажите ставку" }]}
        >
          <InputNumber
            min={0.01}
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
