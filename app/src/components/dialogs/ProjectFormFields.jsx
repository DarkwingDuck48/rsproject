import { DatePicker, Form, Input } from "antd";

/** Формат даты для полей (совпадает с форматом на бэкенде `%Y-%m-%d`). */
export const PROJECT_DATE_FORMAT = "YYYY-MM-DD";

/**
 * Поля формы проекта (название, описание, даты).
 * Общие для NewProjectDialog и EditProjectDialog.
 */
export default function ProjectFormFields() {
  return (
    <>
      <Form.Item
        name="name"
        label="Название"
        rules={[
          {
            required: true,
            whitespace: true,
            message: "Введите название проекта",
          },
        ]}
      >
        <Input placeholder="Название проекта" />
      </Form.Item>
      <Form.Item name="description" label="Описание">
        <Input.TextArea rows={3} placeholder="Описание проекта" />
      </Form.Item>
      <Form.Item
        name="date_start"
        label="Дата начала"
        rules={[{ required: true, message: "Укажите дату начала" }]}
      >
        <DatePicker format={PROJECT_DATE_FORMAT} style={{ width: "100%" }} />
      </Form.Item>
      <Form.Item
        name="date_end"
        label="Дата окончания"
        rules={[{ required: true, message: "Укажите дату окончания" }]}
      >
        <DatePicker format={PROJECT_DATE_FORMAT} style={{ width: "100%" }} />
      </Form.Item>
    </>
  );
}
