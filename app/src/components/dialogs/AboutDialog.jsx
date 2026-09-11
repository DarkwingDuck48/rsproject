import { Button, Modal, Space, Typography } from "antd";
import { GithubOutlined } from "@ant-design/icons";
import { openUrl } from "@tauri-apps/plugin-opener";
import { APP_VERSION, REPO_URL } from "../../lib/constants";

/**
 * Диалог «О программе»: версия и ссылка на репозиторий.
 *
 * @param {Object}   props
 * @param {boolean}  props.open     - Открыт ли диалог.
 * @param {Function} props.onClose  - Закрыть диалог: () => void.
 */
export default function AboutDialog({ open, onClose }) {
  return (
    <Modal
      open={open}
      title="О программе"
      onCancel={onClose}
      footer={[
        <Button key="ok" type="primary" onClick={onClose}>
          ОК
        </Button>,
      ]}
      destroyOnHidden
    >
      <Space direction="vertical" size="small" style={{ width: "100%" }}>
        <Typography.Title level={4} style={{ margin: 0 }}>
          RS Project
        </Typography.Title>
        <Typography.Paragraph type="secondary" style={{ marginBottom: 0 }}>
          Версия {APP_VERSION}
        </Typography.Paragraph>
        <Typography.Paragraph style={{ marginBottom: 0 }}>
          Открытый аналог MS Project на Rust (Tauri v2 + React).
        </Typography.Paragraph>
        <Button
          icon={<GithubOutlined />}
          onClick={() => openUrl(REPO_URL)}
          style={{ alignSelf: "flex-start" }}
        >
          Открыть репозиторий на GitHub
        </Button>
      </Space>
    </Modal>
  );
}
