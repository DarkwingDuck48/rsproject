import { Button, Dropdown, Menu, message } from "antd";
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
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useState } from "react";
import { TAB_ORDER, TABS } from "../../tabs";
import { openProject, saveProject } from "../../lib/api";
import AboutDialog from "../dialogs/AboutDialog";
import CloseProjectDialog from "../dialogs/CloseProjectDialog";
import NewProjectDialog from "../dialogs/NewProjectDialog";

/** Ссылка на репозиторий проекта (используется в меню «Помощь»). */
const REPO_URL = "https://github.com/DarkwingDuck48/rsproject";

/**
 * Верхняя панель приложения: меню «Файл»/«Помощь» и переключатель вкладок.
 * Меню «Файл» управляет проектами (диалоги/команды), «Помощь» — справка.
 *
 * @param {Object}   props
 * @param {string}   props.activeTab          - Текущая активная вкладка (TabKey).
 * @param {Function} props.onTabChange        - Смена вкладки: (tabKey: string) => void.
 * @param {Function} props.onDataChange       - Уведомить приложение об изменении данных.
 */
export default function TopPanel({ activeTab, onTabChange, onDataChange }) {
  /** Открыт ли тот или иной диалог. */
  const [newProjectOpen, setNewProjectOpen] = useState(false);
  const [closeProjectOpen, setCloseProjectOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);

  /**
   * Обработчик пунктов меню «Файл».
   * @param {string} key - Идентификатор пункта меню.
   */
  function onFileMenu(key) {
    switch (key) {
      case "new_project":
        setNewProjectOpen(true);
        break;
      case "open_project":
        handleOpenProject();
        break;
      case "save_project":
        handleSaveProject();
        break;
      case "close_project":
        setCloseProjectOpen(true);
        break;
      case "exit":
        void getCurrentWindow().close();
        break;
    }
  }

  /** Открыть проект через системный диалог (бэкенд) и обновить данные. */
  async function handleOpenProject() {
    try {
      await openProject();
      onDataChange();
    } catch (err) {
      message.error(`Не удалось открыть проект: ${err}`);
    }
  }

  /** Сохранить проект через системный диалог (бэкенд). */
  async function handleSaveProject() {
    try {
      await saveProject();
      message.success("Проект сохранён");
    } catch (err) {
      message.error(`Не удалось сохранить проект: ${err}`);
    }
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
        setAboutOpen(true);
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
    { key: "about", icon: <InfoCircleOutlined />, label: "О программе" },
    { key: "github", icon: <GithubOutlined />, label: "Репозиторий на GitHub" },
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

      <NewProjectDialog
        open={newProjectOpen}
        onClose={() => setNewProjectOpen(false)}
        onCreated={() => {
          onDataChange();
          onTabChange("project");
        }}
      />
      <CloseProjectDialog
        open={closeProjectOpen}
        onClose={() => setCloseProjectOpen(false)}
        onClosed={onDataChange}
      />
      <AboutDialog open={aboutOpen} onClose={() => setAboutOpen(false)} />
    </header>
  );
}
