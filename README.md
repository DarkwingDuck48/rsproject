# RS Project – альтернатива MS Project на Rust

![Testing](https://github.com/yourname/rsproject/actions/workflows/ci.yml/badge.svg)


Управление проектами с задачами, ресурсами, критическим путём и диаграммой Ганта.


## Сборка из исходников

```bash
git clone https://github.com/yourname/rsproject.git
cd rsproject
cargo build --release
./target/release/rsproject

## Возможности

- ✅ Создание проектов, задач и ресурсов
- ✅ Назначение ресурсов с проверкой доступности
- ✅ Иерархия задач (групповые задачи)
- ✅ Расчет критического пути
- ✅ Диаграмма Ганта с подсветкой
- ✅ Сохранение/загрузка в JSON