import { Empty, Typography } from "antd";

/**
 * Вкладка «Проект»: информация о проекте (название, описание, даты).
 * Реализуется в задаче 5.11 — сейчас это каркас-заглушка.
 */
export default function ProjectView() {
  return (
    <section className="view">
      <Typography.Title level={3}>Проект</Typography.Title>
      <Empty description="Информация о проекте появится здесь (5.11)" />
    </section>
  );
}
