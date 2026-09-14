import { createContext, useCallback, useContext, useMemo, useState } from "react";

/**
 * Контекст выделения (задача 5.21).
 *
 * Единый источник истины для выбранных задач/ресурсов: клик в SidePanel и
 * клик в таблице вьюхи меняют одно и то же состояние, поэтому выделение
 * синхронизировано в обе стороны. Состояние хранится per-tab: выделение
 * задачи не пересекается с выделением ресурса.
 */
const SelectionContext = createContext(null);

/**
 * Провайдер выделения — оборачивает всё приложение в `App.jsx`.
 *
 * @param {Object} props
 * @param {React.ReactNode} props.children
 */
export function SelectionProvider({ children }) {
  const [selectedTaskId, setSelectedTaskId] = useState(null);
  const [selectedResourceId, setSelectedResourceId] = useState(null);

  const selectTask = useCallback((id) => setSelectedTaskId(id ?? null), []);
  const selectResource = useCallback(
    (id) => setSelectedResourceId(id ?? null),
    [],
  );

  const value = useMemo(
    () => ({ selectedTaskId, selectTask, selectedResourceId, selectResource }),
    [selectedTaskId, selectTask, selectedResourceId, selectResource],
  );

  return (
    <SelectionContext.Provider value={value}>
      {children}
    </SelectionContext.Provider>
  );
}

/**
 * Хук доступа к выделению.
 * @returns {{ selectedTaskId: ?string, selectTask: (id: ?string) => void,
 *              selectedResourceId: ?string, selectResource: (id: ?string) => void }}
 */
export function useSelection() {
  const ctx = useContext(SelectionContext);
  if (!ctx) {
    throw new Error("useSelection должен использоваться внутри SelectionProvider");
  }
  return ctx;
}
