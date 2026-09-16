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

- [Rust](https://www.rust-lang.org/tools/install) 1.85+ — проект использует edition 2024
- [Node.js](https://nodejs.org/) 20.19+ или 22.12+ — нужен для сборки фронтенда
- Базовые инструменты `rustup`, `cargo`, `npm`

На **Linux** дополнительно нужны системные библиотеки для Tauri v2:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
    librsvg2-dev libxdo-dev libssl-dev
```

### Рекомендуемые инструменты

```bash
# Форматирование кода
rustup component add rustfmt

# Линтер
rustup component add clippy
```

Tauri CLI устанавливать глобально не нужно: локальная версия v2 уже указана в
`app/package.json`, поэтому команды запускаются через `npm run tauri …` из папки `app/`.

## Структура проекта

Проект — Cargo workspace из трёх крейтов: `logic` (бизнес-логика, не зависит от UI),
`app/src-tauri` (десктопное приложение на Tauri v2) и `tools` (инструменты разработки).

```text
rsproject/
├── app/                          # Десктопное приложение (Tauri v2)
│   ├── src-tauri/                # Rust-бэкенд
│   │   ├── src/
│   │   │   ├── main.rs           # Точка входа
│   │   │   ├── lib.rs            # tauri::Builder: плагины и список команд
│   │   │   ├── state.rs          # AppState (Mutex<SingleProjectContainer>)
│   │   │   ├── commands.rs       # Модуль Tauri-команд
│   │   │   ├── commands/         # project, task, resources, utils
│   │   │   ├── dto.rs            # Модуль DTO для фронтенда
│   │   │   └── dto/              # project, task, resources
│   │   ├── capabilities/         # Разрешения плагинов Tauri
│   │   ├── icons/                # Иконки приложения
│   │   ├── build.rs              # Сборка через tauri-build
│   │   ├── tauri.conf.json       # Окно, бандл, идентификатор
│   │   └── Cargo.toml
│   ├── src/                      # React-фронтенд (Vite)
│   │   ├── main.jsx              # Рендер приложения
│   │   ├── App.jsx               # Корневой компонент: вкладки и раскладка
│   │   ├── tabs.jsx              # Реестр вкладок (единый источник истины)
│   │   ├── components/
│   │   │   ├── layout/           # TopPanel, SidePanel, CentralPanel, StatusBar
│   │   │   ├── views/            # ProjectView, TasksView, ResourcesView, GanttView
│   │   │   └── dialogs/          # Диалоги создания/редактирования
│   │   ├── context/              # SelectionContext — общее выделение элементов
│   │   ├── hooks/                # useTheme, useHotkey
│   │   ├── lib/                  # api.js, options.js, format.js, constants.js, errors.js
│   │   └── types/                # JSDoc-типы, описывающие структуры logic
│   ├── index.html                # Точка входа WebView
│   ├── package.json
│   └── vite.config.js
├── logic/                        # Бизнес-логика (без UI-зависимостей)
│   ├── src/
│   │   ├── lib.rs                # Публичный API крейта
│   │   ├── base_structures.rs    # Модуль доменных структур
│   │   ├── base_structures/      # project, tasks, resource, resource_pool,
│   │   │                         # dependencies, project_calendar,
│   │   │                         # project_containers, time_window, traits
│   │   ├── services.rs           # Модуль сервисов
│   │   ├── services/             # scheduler, task_service, resource_service
│   │   └── cust_exceptions.rs    # Пользовательские исключения
│   └── tests/
│       └── integration.rs        # Интеграционные тесты
├── tools/                        # Инструменты разработки (в релиз не входят)
│   ├── src/bin/
│   │   ├── gen_demo_project.rs   # Генератор examples/demo_project.json
│   │   ├── gen_big_project.rs    # Генератор examples/big_project.json
│   │   └── gen_apartment_renovation.rs  # Генератор examples/apartment_renovation.json
│   └── Cargo.toml
├── examples/                     # Готовые проекты для загрузки в приложении
│   ├── demo_project.json
│   ├── big_project.json
│   └── apartment_renovation.json
├── docs/
│   └── screenshots/              # Скриншоты для README
├── .github/                      # CI/CD, шаблоны issue и PR
├── Cargo.toml                    # Workspace: app/src-tauri + logic + tools
├── CHANGELOG.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── README.md
└── LICENSE
```

### Слои приложения

Поток данных при вызове действия из интерфейса:

```text
React-компонент → lib/api.js (invoke) → Tauri-команда (commands/) → сервис (logic/src/services)
                                       ↕ DTO (dto/)
```

- **`logic`** — чистый Rust: доменные структуры и сервисы. Здесь живут все правила
  планирования (расчёт критического пути, аллокация ресурсов, стоимость).
- **`commands/`** — тонкий адаптер между фронтендом и `logic`: разбирает аргументы,
  достаёт проект из `AppState`, конвертирует ошибки в строки.
- **`dto/`** — структуры только для сериализации в фронтенд; они отделяют внутреннюю
  модель `logic` от того, что видит UI.
- **`src/`** — React: вкладки, вьюхи, диалоги. Обращается к бэкенду **только** через
  `invoke()` из `lib/api.js` — прямых вызовов `invoke` в компонентах не должно быть.

### Демо-проекты

Файлы в `examples/` — снапшоты, которые генерируют инструменты из `tools`:

```bash
cargo run -p tools --bin gen_demo_project > examples/demo_project.json
cargo run -p tools --bin gen_big_project > examples/big_project.json
cargo run -p tools --bin gen_apartment_renovation > examples/apartment_renovation.json
```

Генераторы создают случайные идентификаторы (UUID v4), поэтому повторный запуск даёт
файл с другим содержимым: сам проект тот же, но в diff попадут все ID и часть порядка
ключей. Обновляйте эти файлы только вместе с изменением модели данных или самих
генераторов — иначе ревью утонет в шуме.

## Процесс разработки

### 1. Клонирование и настройка

```bash
git clone https://github.com/DarkwingDuck48/rsproject.git
cd rsproject

# Rust-часть: крейты logic и app
cargo build --workspace

# Фронтенд: без него приложение не запустится
cd app
npm ci
```

### 2. Запуск приложения

```bash
cd app
npm run tauri dev
```

Команда поднимает Vite dev-сервер с горячей перезагрузкой фронтенда и пересобирает
Rust-часть при изменениях. Первая сборка занимает несколько минут.

Production-сборка (бинарник и бандлы появятся в `target/release/`):

```bash
cd app
npm run tauri build
```

> Фронтенд собирается **до** компиляции Rust: Tauri встраивает содержимое `app/dist`
> в бинарник. Именно поэтому `cargo build --release` без предварительного
> `npm run build` не даёт рабочего приложения.

### 3. Создание ветки

Создайте ветку от `master`:

```bash
git checkout -b feature/краткое-описание    # для новой функциональности
git checkout -b fix/краткое-описание        # для исправления ошибки
```

### 4. Разработка

Вносите изменения, следуя [стилю кода](#стиль-кода).

### 5. Проверка перед коммитом

```bash
# Компиляция всего workspace
cargo build --workspace

# Запуск всех тестов
cargo test --workspace

# Форматирование
cargo fmt --all

# Проверка линтером
cargo clippy --workspace -- -D warnings

# Сборка фронтенда
cd app && npm run build
```

## Стиль кода

### Общие правила

- Следуйте [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Используйте `rustfmt` для форматирования — конфигурация по умолчанию.
- Все `public` элементы должны быть документированы (`///`).
- Язык комментариев и документации — **русский**.
- Имена переменных, функций, типов — на английском (традиция Rust).

### Именование

| Элемент        | Стиль            | Пример                      |
| -------------- | ---------------- | --------------------------- |
| Модули/крейты  | `snake_case`     | `base_structures`           |
| Типы/Структуры | `UpperCamelCase` | `ProjectCalendar`           |
| Функции/Методы | `snake_case`     | `get_project_tasks()`       |
| Константы      | `UPPER_SNAKE`    | `MAX_TASKS`                 |
| Признаки       | `UpperCamelCase` | `BasicGettersForStructures` |

### Обработка ошибок

- Используйте `anyhow::Result<T>` для прикладного кода.
- Используйте `thiserror` для библиотечных ошибок (крейт `logic`).
- Избегайте `unwrap()` и `expect()` в production-коде — возвращайте `Result`.
- В Tauri-командах ошибки конвертируются в `String` (`Result<T, String>`), потому что
  именно строку получает фронтенд.

### Сериализация

- Используйте `serde` с `derive` для `Serialize`/`Deserialize`.
- Поля в JSON сохраняют имена Rust-структур — `snake_case` (переименование не настроено).
  Файлы проектов совместимы с этим форматом, поэтому менять имена полей можно только
  вместе с миграцией.
- На границе `invoke()` (Tauri v2) верхнеуровневые аргументы команды — `camelCase`
  (макрос `#[tauri::command]` сам конвертирует `snake_case`-параметры: `date_start` →
  `dateStart`), а поля вложенных DTO остаются `snake_case`.

### Зависимости

- Новые внешние крейты и npm-пакеты должны быть обоснованы в PR.
- Предпочитайте крейты с лицензией Apache 2.0 или MIT.
- Rust-зависимости добавляйте в корневой `Cargo.toml` в секцию `[workspace.dependencies]`,
  а в крейтах подключайте через `workspace = true`.

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
- [ ] `cd app && npm run build` — фронтенд собирается без ошибок
- [ ] Добавлены тесты для новой функциональности
- [ ] Обновлена документация (если необходимо)
- [ ] Изменения проверены на целевой платформе

### После отправки

- CI-пайплайн (`.github/workflows/ci.yml`) автоматически запустит сборку и тесты.
- Мейнтейнер проведёт код-ревью. Будьте готовы к обсуждению и правкам.
- После одобрения PR будет влит в `master`.

---

Если у вас остались вопросы — создайте [обсуждение](https://github.com/DarkwingDuck48/rsproject/discussions).
