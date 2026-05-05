# RS Project – альтернатива MS Project на Rust

![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)
![Testing](https://github.com/DarkwingDuck48/rsproject/actions/workflows/ci.yml/badge.svg)
![Release](https://github.com/DarkwingDuck48/rsproject/actions/workflows/release.yml/badge.svg)


Управление проектами с задачами, ресурсами, критическим путём и диаграммой Ганта.

## Участие в разработке

Мы приветствуем вклад в проект! Ознакомьтесь с [руководством для участников](CONTRIBUTING.md) и [кодексом поведения](CODE_OF_CONDUCT.md).

## Версионирование

Проект следует принципам [Semantic Versioning](https://semver.org/spec/v2.0.0.html) (SemVer).

Номер версии имеет формат `MAJOR.MINOR.PATCH`:

| Компонент | Когда изменяется                                      | Пример          |
|-----------|-------------------------------------------------------|-----------------|
| **MAJOR** | Несовместимые изменения (breaking changes)            | `1.0.0 → 2.0.0` |
| **MINOR** | Новая функциональность с обратной совместимостью      | `0.1.0 → 0.2.0` |
| **PATCH** | Исправления ошибок с обратной совместимостью          | `0.1.0 → 0.1.1` |

До версии `1.0.0` интерфейс может меняться — следите за [CHANGELOG](CHANGELOG.md) при обновлении.

Текущая версия: **0.1.0**

## Скачать

Последнюю версию можно скачать на [странице релизов](https://github.com/DarkwingDuck48/rsproject/releases/latest).

## Для пользователей MacOS

Так как ПО распространяется бесплатно, то после скачивания возможно появление предупреждения от Apple о том, что невозможно проверить приложение.
Для того, чтобы запустить RS Project, нужно выполнить 2 команды в терминале

**Разрешить выполнение файла**

```bash
chmod +x /путь/к/скаченному/rsproject
```

**Убрать предупреждение от Apple**
```bash
xattr -d com.apple.quarantine /путь/к/rsproject
```


## Сборка из исходников

```bash
git clone https://github.com/DarkwingDuck48/rsproject.git
cd rsproject
cargo build --release
./target/release/rsproject
```

## Возможности

✅ Создание проектов, задач и ресурсов

✅ Назначение ресурсов с проверкой доступности

✅ Иерархия задач (групповые задачи)

✅ Расчет критического пути

✅ Диаграмма Ганта с подсветкой

✅ Сохранение/загрузка в JSON

## В планах

🤔 Добавить возможность открытия нескольких проектов и объединения их в один

🤔 Загрузка и экспорт в формат MS Project для обратной совместимости


## Обзор интерфейса

### Экран информации по проекту

![main_screen](/docs/screenshots/main_screen.png)

### Экран задач

![tasks_screen](/docs/screenshots/tasks_screen.png)

### Экран ресурсов

![tasks_screen](/docs/screenshots/resource_screen.png)

### Диаграмма Ганта и критический путь проекта

![tasks_screen](/docs/screenshots/gantt_with_critical.png)
