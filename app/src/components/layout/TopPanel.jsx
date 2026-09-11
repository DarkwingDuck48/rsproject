import { Button, Dropdown, Menu } from "antd";
import {
  CloseOutlined,
  FileAddOutlined,
  FolderOpenOutlined,
  GithubOutlined,
  InfoCircleOutlined,
  LogoutOutlined,
  SaveOutlined,
} from "@ant-design/icons";
import { openUrl } from "@tauri-apps/plugin-opener";
import { TAB_ORDER, TABS } from "../../tabs";

/** Ссылка на репозиторий проекта (используется в меню «Помощь»). */
const REPO_URL = "https://github.com/DarkwingDuck48/rsproject";

/**
 * Верхняя панель приложения: меню «Файл»/«Помощь» и переключатель вкладок.
 *
 * @param {Object}   props
 * @param {string}   props.activeTab   - Текущая активная вкладка (TabKey).
 * @param {Function} props.onTabChange - Колбэк смены вкладки: (tabKey: string) => void
 */
export default function TopPanel({ activeTab, onTabChange }) {
  /**
   * Обработчик пунктов меню «Файл».
   * Реальная логика появится вместе с диалогами в задаче 5.14.
   * @param {string} _key - Идентификатор пункта меню.
   */
  function onFileMenu(_key) {
    // TODO(5.14): открытие соответствующих диалогов.
  }

  /**
   * Обработчик пунктов меню «Помощь».
   * @param {string} key - Идентификатор пункта меню.
   */
  function onHelpMenu(key) {
    switch (key) {
      case "github":
        openUrl(REPO_URL);
        break;
      case "about":
        // TODO(5.14): открытие AboutDialog.
        break;
    }
  }

  const fileMenuItems = [
    { key: "new_project", icon: <FileAddOutlined />, label: "Новый проект" },
    {
      key: "open_project",
      icon: <FolderOpenOutlined />,
      label: "Открыть проект",
    },
    { key: "save_project", icon: <SaveOutlined />, label: "Сохранить проект" },
    { key: "close_project", icon: <CloseOutlined />, label: "Закрыть проект" },
    { type: "divider" },
    { key: "exit", icon: <LogoutOutlined />, label: "Выход" },
  ];

  const helpMenuItems = [
    {
      key: "about",
      icon: <InfoCircleOutlined />,
      label: "О программе",
    },
    {
      key: "github",
      icon: <GithubOutlined />,
      label: "Репозиторий на GitHub",
    },
  ];

  const tabMenuItems = TAB_ORDER.map((key) => ({
    key,
    icon: TABS[key].icon,
    label: TABS[key].label,
  }));

  return (
    <header className="app-top-panel">
      <div className="app-top-panel__brand">
        <span className="app-top-panel__brand-name">RS Project</span>
      </div>

      <Dropdown
        menu={{ items: fileMenuItems, onClick: ({ key }) => onFileMenu(key) }}
        trigger={["click"]}
      >
        <Button type="text" className="app-top-panel__menu-button">
          Файл
        </Button>
      </Dropdown>

      <Dropdown
        menu={{ items: helpMenuItems, onClick: ({ key }) => onHelpMenu(key) }}
        trigger={["click"]}
      >
        <Button type="text" className="app-top-panel__menu-button">
          Помощь
        </Button>
      </Dropdown>

      <Menu
        className="app-top-panel__tabs"
        mode="horizontal"
        items={tabMenuItems}
        selectedKeys={[activeTab]}
        onClick={({ key }) => onTabChange(key)}
      />
    </header>
  );
}
