/**
 * @fileoverview Хук глобальных горячих клавиш — задача 5.17.
 *
 * Регистрирует слушатель `keydown` на `window`, поэтому срабатывает
 * независимо от того, какая панель/вкладка в фокусе.
 * Для Delete важен флаг `ignoreTyping`: клавиша не должна удалять
 * задачу/ресурс, пока пользователь печатает в поле ввода (например,
 * в диалоге).
 */

import { useEffect, useRef } from "react";

/**
 * true, если событие пришло из поля ввода (input/textarea/select/contentEditable).
 * @param {KeyboardEvent} event
 * @returns {boolean}
 */
function isTypingTarget(event) {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName?.toLowerCase();
  return tag === "input" || tag === "textarea" || tag === "select";
}

/**
 * Регистрирует глобальный хоткей.
 *
 * Обработчик всегда вызывается «свежей» версией (через ref): слушатель
 * регистрируется один раз, а актуальная функция подставляется на каждом
 * рендере, поэтому хоткей никогда не замыкает устаревшие данные.
 *
 * @param {string}   key         - Код клавиши (KeyboardEvent.key), напр. "Delete" или "n".
 * @param {Function} handler     - Обработчик.
 * @param {Object}   [options]
 * @param {boolean}  [options.ctrl]         - Требовать Ctrl/Cmd (без модификатора не сработает).
 * @param {boolean}  [options.ignoreTyping] - Не срабатывать при вводе текста.
 */
export function useHotkey(key, handler, options = {}) {
  const { ctrl = false, ignoreTyping = false } = options;

  // Актуальный обработчик перезаписывается после каждого рендера.
  const handlerRef = useRef(handler);
  useEffect(() => {
    handlerRef.current = handler;
  });

  useEffect(() => {
    function onKeyDown(event) {
      if (event.key.toLowerCase() !== key.toLowerCase()) return;
      if (ctrl && !(event.ctrlKey || event.metaKey)) return;
      if (ignoreTyping && isTypingTarget(event)) return;
      event.preventDefault();
      handlerRef.current();
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [key, ctrl, ignoreTyping]);
}
