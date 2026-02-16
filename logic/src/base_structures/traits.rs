use crate::{
    Project,
    base_structures::{
        project_calendar::ProjectCalendar, resource::Resource, resource_pool::AllocationRequest,
    },
};
use anyhow::Result;
use chrono::{DateTime, TimeDelta, Utc};
use uuid::Uuid;

pub trait ResourcePool {
    fn allocate(&mut self, request: AllocationRequest, calendar: &ProjectCalendar) -> Result<()>;
    fn deallocate(&mut self, allocation_id: Uuid) -> Result<()>;
    fn add_resource(&mut self, resource: Resource) -> Result<()>;
    fn remove_resource(&mut self, id: &Uuid) -> Result<()>;
}

pub trait ProjectContainer {
    fn add_project(&mut self, project: Project) -> Result<()>;
    fn get_project(&self, id: &Uuid) -> Option<&Project>;
    // общий пул ресурсов
    fn resource_pool(&self) -> &dyn ResourcePool;
    fn resource_pool_mut(&mut self) -> &mut dyn ResourcePool;

    // Доступ к календарю проекта
    fn calendar(&self, project_id: &Uuid) -> Option<&ProjectCalendar>;
}

pub trait BasicGettersForStructures {
    fn get_id(&self) -> &Uuid;
    fn get_date_start(&self) -> &DateTime<Utc>;
    fn get_date_end(&self) -> &DateTime<Utc>;
    fn get_duration(&self) -> &TimeDelta;
}
