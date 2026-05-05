# Участие в разработке RS Project

Спасибо за интерес к проекту! Этот документ описывает, как внести свой вклад в разработку RS Project.

## Оглавление

- [Кодекс поведения](#кодекс-поведения)
- [Прежде чем начать](#прежде-чем-начать)
- [Как сообщить об ошибке](#как-сообщить-об-ошибке)
- [Как предложить новую возможность](#как-предложить-новую-возможность)
- [Рабочее окружение](#рабочее-окружение)
- [Структура проекта](#структура-проекта)
- [Процесс разработки](#процесс-разработки)
- [Стиль кода](#стиль-кода)
- [Тестирование](#тестирование)
- [Pull Request](#pull-request)

---

## Кодекс поведения

Участники проекта обязуются следовать [Кодексу поведения](CODE_OF_CONDUCT.md). Пожалуйста, прочитайте его перед участием.

## Прежде чем начать

1. Проверьте [список открытых issues](https://github.com/DarkwingDuck48/rsproject/issues) — возможно, над задачей уже кто-то работает.
2. Для крупных изменений создайте [feature request](https://github.com/DarkwingDuck48/rsproject/issues/new?template=feature_request.yml) и обсудите идею до начала реализации.
3. Если нашли ошибку — создайте [bug report](https://github.com/DarkwingDuck48/rsproject/issues/new?template=bug_report.yml).

## Как сообщить об ошибке

Используйте шаблон **Bug Report** при создании issue. Обязательно укажите:

- Версию RS Project
- Операционную систему
- Шаги для воспроизведения
- Ожидаемое и фактическое поведение
- Логи или вывод консоли (если есть)

## Как предложить новую возможность

Используйте шаблон **Feature Request** при создании issue. Опишите:

- Какую проблему решает предложение
- Как вы видите решение
- Альтернативные подходы (если рассматривали)

## Рабочее окружение

### Требования

- [Rust](https://www.rust-lang.org/tools/install) (stable, последней версии)
- Базовые инструменты `rustup`, `cargo`

### Рекомендуемые инструменты

```bash
# Форматирование кода
rustup component add rustfmt

# Линтер
rustup component add clippy
```

## Структура проекта

```
rsproject/
├── app/                  # GUI-приложение (интерфейс)
│   ├── src/
│   │   ├── main.rs       # Точка входа
│   │   ├── lib.rs        # Публичный API приложения
│   │   └── app/
│   │       ├── app_impl.rs   # Основная логика приложения
│   │       ├── state.rs      # Состояние приложения
│   │       ├── theme.rs      # Тема оформления
│   │       ├── ui/           # Компоненты пользовательского интерфейса
│   │       ├── views/        # Экраны: проект, задачи, ресурсы, Гант
│   │       ├── dialogs/      # Диалоговые окна
│   │       └── handlers/     # Обработчики действий
│   └── examples/         # Демонстрационные примеры
├── logic/                # Бизнес-логика (ядро)
│   ├── src/
│   │   ├── lib.rs            # Публичный API ядра
│   │   ├── base_structures/  # Основные структуры данных
│   │   │   ├── project.rs, tasks.rs, resource.rs
│   │   │   ├── dependencies.rs, time_window.rs
│   │   │   ├── resource_pool.rs, project_calendar.rs
│   │   │   └── project_containers.rs, traits.rs
│   │   ├── services/         # Сервисы планирования
│   │   │   ├── scheduler.rs
│   │   │   ├── resource_service.rs
│   │   │   └── task_service.rs
│   │   └── cust_exceptions.rs
│   └── tests/
│       └── integration.rs    # Интеграционные тесты
├── docs/                 # Документация и скриншоты
└── .github/              # CI/CD и шаблоны
```

## Процесс разработки

### 1. Клонирование и настройка

```bash
git clone https://github.com/DarkwingDuck48/rsproject.git
cd rsproject
cargo build --workspace
```

### 2. Создание ветки

Создайте ветку от `master`:

```bash
git checkout -b feature/краткое-описание    # для новой функциональности
git checkout -b fix/краткое-описание        # для исправления ошибки
```

### 3. Разработка

Вносите изменения, следуя [стилю кода](#стиль-кода).

### 4. Проверка перед коммитом

```bash
# Компиляция всего workspace
cargo build --workspace

# Запуск всех тестов
cargo test --workspace

# Форматирование
cargo fmt --all

# Проверка линтером
cargo clippy --workspace -- -D warnings
```

## Стиль кода

### Общие правила

- Следуйте [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Используйте `rustfmt` для форматирования — конфигурация по умолчанию.
- Все `public` элементы должны быть документированы (`///`).
- Язык комментариев и документации — **русский**.
- Имена переменных, функций, типов — на английском (традиция Rust).

### Именование

| Элемент        | Стиль          | Пример                    |
|----------------|----------------|---------------------------|
| Модули/крейты  | `snake_case`   | `base_structures`         |
| Типы/Структуры | `UpperCamelCase` | `ProjectCalendar`      |
| Функции/Методы | `snake_case`   | `get_project_tasks()`     |
| Константы      | `UPPER_SNAKE`  | `MAX_TASKS`               |
| Признаки       | `UpperCamelCase` | `BasicGettersForStructures` |

### Обработка ошибок

- Используйте `anyhow::Result<T>` для прикладного кода.
- Используйте `thiserror` для библиотечных ошибок (крейт `logic`).
- Избегайте `unwrap()` и `expect()` в production-коде — возвращайте `Result`.

### Сериализация

- Используйте `serde` с `derive` для `Serialize`/`Deserialize`.
- Поля хранятся в `camelCase` при экспорте JSON (если настроено).

### Зависимости

- Новые внешние крейты должны быть обоснованы в PR.
- Предпочитайте крейты с лицензией Apache 2.0 или MIT.
- Добавляйте зависимости в корневой `Cargo.toml` в секцию `[workspace.dependencies]`.

## Тестирование

### Unit-тесты

Размещайте внутри файлов с исходным кодом в модуле `tests`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

### Интеграционные тесты

Размещайте в `logic/tests/` или `app/tests/`.

### Запуск тестов

```bash
# Все тесты
cargo test --workspace

# Только для крейта logic
cargo test -p logic

# Только для крейта app
cargo test -p app

# С выводом (stdout/stderr visible)
cargo test --workspace -- --nocapture

# Конкретный тест
cargo test -p logic -- test_create_empty_project
```

## Pull Request

### Подготовка PR

1. Убедитесь, что все пункты чеклиста выполнены.
2. Заполните шаблон PR.
3. Привяжите PR к issue (если есть): `Closes #номер`.

### Чеклист перед отправкой

- [ ] `cargo build --workspace` — без ошибок и предупреждений
- [ ] `cargo test --workspace` — все тесты проходят
- [ ] `cargo fmt --all` — код отформатирован
- [ ] `cargo clippy --workspace -- -D warnings` — нет замечаний линтера
- [ ] Добавлены тесты для новой функциональности
- [ ] Обновлена документация (если необходимо)
- [ ] Изменения проверены на целевой платформе

### После отправки

- CI-пайплайн (`.github/workflows/ci.yml`) автоматически запустит сборку и тесты.
- Мейнтейнер проведёт код-ревью. Будьте готовы к обсуждению и правкам.
- После одобрения PR будет влит в `master`.

---

Если у вас остались вопросы — создайте [обсуждение](https://github.com/DarkwingDuck48/rsproject/discussions).
