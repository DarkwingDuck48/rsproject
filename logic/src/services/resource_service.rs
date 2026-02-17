use crate::base_structures::{ExceptionPeriod, ProjectContainer, RateMeasure, Resource};
use anyhow::Result;
use uuid::Uuid;
pub struct ResourceService<'a, C: ProjectContainer> {
    container: &'a mut C,
}

impl<'a, C: ProjectContainer> ResourceService<'a, C> {
    pub fn new(container: &'a mut C) -> Self {
        Self { container }
    }

    pub fn create_resource(
        &mut self,
        name: impl Into<String>,
        rate: f64,
        measure: RateMeasure,
    ) -> Result<Resource> {
        Resource::new(name.into(), rate, measure)
    }

    pub fn add_resource(&mut self, resource: Resource) -> Result<()> {
        self.container.resource_pool_mut().add_resource(resource)
    }

    pub fn list_resources(&self) -> Vec<&Resource> {
        self.container.resource_pool().get_resources()
    }

    pub fn add_unavailable_period(
        &mut self,
        resource_id: Uuid,
        exception_period: ExceptionPeriod,
    ) -> Result<()> {
        match self
            .container
            .resource_pool_mut()
            .get_mut_resource_by_uuid(resource_id)
        {
            Some(r) => {
                let _: () = r.add_unavailable_period(exception_period);
                Ok(())
            }
            None => Err(anyhow::Error::msg("Resource not found in poll")),
        }
    }

    /// Суммарная занятость ресурса
    pub fn get_resource_utilization(&self, resource_id: Uuid) -> f64 {
        self.container
            .resource_pool()
            .get_resource_existing_allocations(&resource_id)
            .iter()
            .map(|ra| *ra.get_engagement_rate())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_structures::{
        BasicGettersForStructures, ExceptionPeriod, ExceptionType, Project, RateMeasure,
        SingleProjectContainer, TimeWindow,
    };
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_resource_pool() {
        let mut container = SingleProjectContainer::new();
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap();
        let project = Project::new("Test", "Desc", start, end).unwrap();
        let project_id = *project.get_id();
        container.add_project(project).unwrap();

        let mut resource_service = ResourceService::new(&mut container);
        let new_resource = resource_service
            .create_resource("TestRes", 1000.0, RateMeasure::Hourly)
            .unwrap();

        let new_resorce_uuid = new_resource.id;
        assert_eq!(new_resource.name, "TestRes");

        assert!(resource_service.add_resource(new_resource).is_ok());

        let vacations = ExceptionPeriod {
            exception_type: ExceptionType::Vacation,
            period: TimeWindow {
                date_start: Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap(),
                date_end: Utc.with_ymd_and_hms(2025, 3, 14, 0, 0, 0).unwrap(),
            },
        };
        assert!(
            resource_service
                .add_unavailable_period(new_resorce_uuid, vacations)
                .is_ok()
        );
        let resources_list = resource_service.list_resources();
        assert_eq!(resources_list.len(), 1);

        let resource_utilization = resource_service.get_resource_utilization(resources_list[0].id);
        assert_eq!(resource_utilization, 0.0);
        assert!(!resources_list[0].is_available(
            &TimeWindow {
                date_start: Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap(),
                date_end: Utc.with_ymd_and_hms(2025, 3, 14, 0, 0, 0).unwrap(),
            },
            resource_service.container.calendar(&project_id).unwrap(),
        ))
    }
}
