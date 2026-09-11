import { Empty, Typography } from "antd";

/**
 * Вкладка «Диаграмма Ганта»: диаграмма с подсветкой критического пути.
 * Реализуется в задаче 5.15 — сейчас это каркас-заглушка.
 */
export default function GanttView() {
  return (
    <section className="view">
      <Typography.Title level={3}>Диаграмма Ганта</Typography.Title>
      <Empty description="Диаграмма Ганта появится здесь (5.15)" />
    </section>
  );
}
