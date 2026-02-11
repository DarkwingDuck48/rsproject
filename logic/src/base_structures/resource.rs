use std::collections::HashMap;

/// Глобальные ресурсы.
///
/// Идея в том, что ресурс - это глобальная структура, которая одинакова между разными проектами
/// и ее UUID не зависит от проекта.
/// Однако, с использованием глобальных ресурсов мы можем поймать ситуацию, когда ресурсы будут не совпадать в проектах, созданных отдельно
/// Поэтому нужно реализовать следующую логику:
/// 1. В рамках запуска программы создается глобальный реестр ресурсов
/// 2. В каждом проекте есть локальная версия ресурсов, которая отвечает за используемые в проекте ресурсы из глобальных
/// 3. Если открыто несколько проектов - то нужно выполнить мэппинг локальных ресурсов в глобальные реестр - таким образом мы сможем выполнить оптимизацию всех ресурсов.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Тип ресурса.
#[derive(Serialize, Deserialize, Debug, Default)]
pub enum ResurceTypes {
    #[default]
    Human,
    Material,
}

type ProjectId = Uuid;
type LocalResourceId = Uuid;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GlobalResourceRegister {
    pub id: Uuid,
    pub name: String,
    // Тут храним связь между проектом и локальным ресурсом, чтобы в дальнейшем мы могли его искать и получать нужную информацию
    pub register: HashMap<ProjectId, LocalResourceId>,
    pub rate: f64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Resource {
    pub id: Uuid,
    pub name: String,
    pub res_type: ResurceTypes,
    pub rate: f64,
}

impl Resource {
    pub fn new(name: impl Into<String>, res_type: ResurceTypes, rate: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            res_type,
            rate,
        }
    }
}
