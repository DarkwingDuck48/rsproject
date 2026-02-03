use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Тип ресурса.
#[derive(Serialize, Deserialize, Debug)]
pub enum ResurceTypes {
    Human,
    Material,
}

/// Настройка ресурса. Будет настраиваться внутри каждого проекта отдельно.
/// План такой - мы в любой момент можем создать новый ресурс и задать ему тип и ставку
#[derive(Serialize, Deserialize, Debug)]
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
