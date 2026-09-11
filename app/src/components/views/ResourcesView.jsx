import { Empty, Typography } from "antd";

/**
 * Вкладка «Ресурсы»: таблица ресурсов и периоды недоступности.
 * Реализуется в задаче 5.13 — сейчас это каркас-заглушка.
 */
export default function ResourcesView() {
  return (
    <section className="view">
      <Typography.Title level={3}>Ресурсы</Typography.Title>
      <Empty description="Таблица ресурсов появится здесь (5.13)" />
    </section>
  );
}
