import { Empty, Typography } from "antd";

/**
 * Вкладка «Задачи»: дерево задач с иерархией.
 * Реализуется в задаче 5.12 — сейчас это каркас-заглушка.
 */
export default function TasksView() {
  return (
    <section className="view">
      <Typography.Title level={3}>Задачи</Typography.Title>
      <Empty description="Дерево задач появится здесь (5.12)" />
    </section>
  );
}
