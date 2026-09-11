import { Button, Modal, Typography, message } from "antd";
import { CloseOutlined, SaveOutlined } from "@ant-design/icons";
import { closeProject, saveProject } from "../../lib/api";

/**
 * Диалог подтверждения закрытия проекта.
 *
 * Предлагает сохранить изменения (кнопка «Сохранить и закрыть») или
 * закрыть без сохранения («Закрыть без сохранения»).
 *
 * @param {Object}   props
 * @param {boolean}  props.open     - Открыт ли диалог.
 * @param {Function} props.onClose  - Закрыть диалог без действия: () => void.
 * @param {Function} props.onClosed - Проект закрыт: () => void (для обновления данных).
 */
export default function CloseProjectDialog({ open, onClose, onClosed }) {
  /** Закрыть проект без сохранения. */
  async function handleCloseWithoutSave() {
    try {
      await closeProject();
      onClose();
      onClosed();
    } catch (err) {
      message.error(`Не удалось закрыть проект: ${err}`);
    }
  }

  /** Сохранить проект и закрыть. */
  async function handleSaveAndClose() {
    try {
      await saveProject();
    } catch (err) {
      // Пользователь мог отменить системный диалог сохранения
      // (бэкенд возвращает Ok при отмене, поэтому сюда попадают реальные ошибки)
      message.error(`Не удалось сохранить проект: ${err}`);
      return;
    }
    await handleCloseWithoutSave();
  }

  return (
    <Modal
      open={open}
      title="Закрыть проект"
      onCancel={onClose}
      footer={
        <>
          <Button onClick={onClose}>Отмена</Button>
          <Button danger onClick={handleCloseWithoutSave}>
            Закрыть без сохранения
          </Button>
          <Button
            type="primary"
            icon={<SaveOutlined />}
            onClick={handleSaveAndClose}
          >
            Сохранить и закрыть
          </Button>
        </>
      }
      destroyOnHidden
    >
      <Typography.Paragraph style={{ marginBottom: 0 }}>
        Текущий проект будет закрыт. Несохранённые изменения могут быть
        потеряны. Сохранить проект перед закрытием?
      </Typography.Paragraph>
    </Modal>
  );
}
