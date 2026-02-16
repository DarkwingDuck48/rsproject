use std::collections::HashMap;

use uuid::Uuid;

use crate::base_structures::{
    project_calendar::ProjectCalendar, resource::Resource, time_window::TimeWindow,
    traits::ResourcePool,
};

#[derive(Clone, Copy, Debug)]
pub struct AllocationRequest {
    resource_id: Uuid,
    task_id: Uuid,
    project_id: Uuid,
    engagement_rate: f64,
    time_window: TimeWindow,
}

impl AllocationRequest {
    pub fn new(
        resource_id: Uuid,
        task_id: Uuid,
        project_id: Uuid,
        engagement_rate: f64,
        time_window: TimeWindow,
    ) -> Self {
        Self {
            resource_id,
            task_id,
            project_id,
            engagement_rate,
            time_window,
        }
    }
}

pub struct AllocationQueryResult<'a> {
    allocations_list: Vec<&'a ResourceAllocation>,
}

impl<'a> AllocationQueryResult<'a> {
    pub fn check_correct_timewindow(self, allocation_request: &AllocationRequest) -> bool {
        let overlapping_allocations: Vec<&&ResourceAllocation> = self
            .allocations_list
            .iter()
            .filter(|ra| ra.time_window.overlaps(&allocation_request.time_window))
            .collect();

        let total_engagement: f64 = overlapping_allocations
            .iter()
            .map(|ra| *ra.get_engagement_rate())
            .sum();

        total_engagement + allocation_request.engagement_rate <= 1.0
    }
    pub fn len(&self) -> usize {
        self.allocations_list.len()
    }
    pub fn is_empty(&self) -> bool {
        self.allocations_list.is_empty()
    }
}

// Объект для описания назначения одного из ресурсов на задачу
#[derive(Default, Debug)]
pub struct ResourceAllocation {
    id: Uuid,
    resource_id: Uuid,
    task_id: Uuid,
    project_id: Uuid,
    engagement_rate: f64,
    time_window: TimeWindow,
}

impl ResourceAllocation {
    pub fn new(request: AllocationRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            resource_id: request.resource_id,
            task_id: request.task_id,
            project_id: request.project_id,
            time_window: request.time_window,
            engagement_rate: request.engagement_rate,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_engagement_rate(&self) -> &f64 {
        &self.engagement_rate
    }
}

#[derive(Default, Debug)]
pub struct LocalResourcePool {
    resources: HashMap<Uuid, Resource>,
    allocations: HashMap<Uuid, ResourceAllocation>,
}

impl LocalResourcePool {
    fn check_resource_exists(&self, resource_id: &Uuid) -> bool {
        self.resources.contains_key(resource_id)
    }

    pub fn get_resource_by_name(&self, find_name: String) -> Option<&Resource> {
        self.resources.values().find(|r| r.name == find_name)
    }

    /// Функция должна проверить, что ресурс можно корректно назначить на
    pub fn get_resource_existing_allocations(
        &self,
        resource_id: &Uuid,
    ) -> Vec<&ResourceAllocation> {
        self.allocations
            .values()
            .filter(|a| &a.resource_id == resource_id)
            .collect()
    }

    /// Несколько проверок перед назначением ресурса на задачу в пуле
    /// 1. Ресурс с таким ID существует в пуле
    fn check_allocation_correct(
        &self,
        request: &AllocationRequest,
        calendar: &ProjectCalendar,
    ) -> anyhow::Result<()> {
        let resource = self
            .resources
            .get(&request.resource_id)
            .ok_or_else(|| anyhow::Error::msg("Resource not found"))?;

        if !resource.is_available(&request.time_window, calendar) {
            return Err(anyhow::Error::msg(
                "Resource is not available during requested time (vacation, non-working hours, etc)",
            ));
        }

        let existing_allocation_on_resource =
            self.get_resource_existing_allocations(&request.resource_id);

        // Ресурс есть в пуле и у него еще нет никаких аллокаций - можем смело добавлять.
        if existing_allocation_on_resource.is_empty() {
            return Ok(());
        }

        let aqr = AllocationQueryResult {
            allocations_list: existing_allocation_on_resource,
        };

        // Нашли существующие аллокации - нужно проверить, что
        // 1. У ресуса есть свободное окно, чтобы заниматься работой
        // 2. Если окна занятости пересекаются - сумма всех engagement_rate у всех пересекающихся аллокаций должна быть <= 1.0
        if !aqr.check_correct_timewindow(request) {
            return Err(anyhow::Error::msg(
                "This allocation can't be created, because Resoure will be utilized more than 100%",
            ));
        }

        Ok(())
    }
}

impl ResourcePool for LocalResourcePool {
    fn allocate(
        &mut self,
        request: AllocationRequest,
        calendar: &ProjectCalendar,
    ) -> anyhow::Result<()> {
        match self.check_allocation_correct(&request, calendar) {
            Ok(()) => {
                let allocation = ResourceAllocation::new(request);
                self.allocations.insert(allocation.get_id(), allocation);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn deallocate(&mut self, allocation_id: Uuid) -> anyhow::Result<()> {
        let alocation = self.allocations.remove(&allocation_id);
        match alocation {
            Some(_) => Ok(()),
            None => Err(anyhow::Error::msg("This allocation not found")),
        }
    }

    fn add_resource(&mut self, resource: Resource) -> anyhow::Result<()> {
        self.resources.insert(resource.id, resource);
        Ok(())
    }

    fn remove_resource(&mut self, id: &Uuid) -> anyhow::Result<()> {
        match self.resources.contains_key(id) {
            true => {
                self.resources.remove(id);
                Ok(())
            }
            false => Err(anyhow::Error::msg(format!(
                "No resource with id {} in LocalPool",
                id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};

    use crate::base_structures::{
        project_calendar::ProjectCalendar,
        resource::{RateMeasure, Resource},
        resource_pool::{AllocationRequest, LocalResourcePool},
        time_window::TimeWindow,
        traits::ResourcePool,
    };

    #[test]
    fn test_deallocate() {
        let mut lrp = LocalResourcePool::default();
        let project_calendar = ProjectCalendar::default();
        let resource = Resource::new(String::from("Test"), 1000.0, RateMeasure::Hourly)
            .expect("Can't create resource");
        lrp.add_resource(resource.clone()).unwrap();
        let project_id = uuid::Uuid::new_v4();

        let allocation_request = AllocationRequest::new(
            resource.id,
            uuid::Uuid::new_v4(),
            project_id,
            0.8,
            TimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            )
            .unwrap(),
        );

        assert!(lrp.allocate(allocation_request, &project_calendar).is_ok());

        let al = lrp.get_resource_existing_allocations(&resource.id);
        let al_id = al[0];

        assert!(lrp.deallocate(al_id.get_id()).is_ok())
    }
    #[test]
    fn test_allocation_check() {
        let mut lrp = LocalResourcePool::default();
        let project_calendar = ProjectCalendar::default();
        let resource = Resource::new(String::from("Test"), 1000.0, RateMeasure::Hourly)
            .expect("Can't create resource");

        let project_id = uuid::Uuid::new_v4();

        let allocation_request = AllocationRequest::new(
            resource.id,
            uuid::Uuid::new_v4(),
            project_id,
            0.8,
            TimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            )
            .unwrap(),
        );
        assert!(!lrp.check_resource_exists(&resource.id));
        // Нельзя назначить, пока ресурс не в пуле
        assert!(lrp.allocate(allocation_request, &project_calendar).is_err());

        lrp.add_resource(resource.clone()).unwrap();
        assert!(lrp.allocate(allocation_request, &project_calendar).is_ok());

        let allocation_request2 = AllocationRequest::new(
            resource.id,
            uuid::Uuid::new_v4(),
            project_id,
            0.1,
            TimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            )
            .unwrap(),
        );
        assert!(lrp.allocate(allocation_request2, &project_calendar).is_ok());

        let allocation_request3 = AllocationRequest::new(
            resource.id,
            uuid::Uuid::new_v4(),
            project_id,
            0.2,
            TimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap(),
            )
            .unwrap(),
        );
        assert!(
            lrp.allocate(allocation_request3, &project_calendar)
                .is_err()
        );
    }

    #[test]
    fn test_resource_measure_converter() {
        let resource = Resource::new(String::from("Test"), 1000.0, RateMeasure::Hourly)
            .expect("Can't create resource");
        assert_eq!(resource.get_base_rate(), &1000.0);
        assert_eq!(
            resource.get_converted_rate(crate::base_structures::resource::RateMeasure::Daily),
            8000.0
        );
        assert_eq!(
            resource.get_converted_rate(crate::base_structures::resource::RateMeasure::Monthly),
            22000.0
        );
    }

    #[test]
    fn test_timewindows() {
        let date_first_start: DateTime<Utc> = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date_first_end: DateTime<Utc> = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

        let date_second_start: DateTime<Utc> = Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();
        let date_second_end: DateTime<Utc> = Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap();

        let timewindow1 = TimeWindow::new(date_first_start, date_first_end).unwrap();
        let timewindow2 = TimeWindow::new(date_second_start, date_second_end).unwrap();

        assert!(timewindow1.overlaps(&timewindow2));
    }

    #[test]
    fn test_get_allocation_by_resource() {
        let project_calendar = ProjectCalendar::default();
        let resource = Resource::new(String::from("Test"), 1000.0, RateMeasure::Hourly)
            .expect("Can't create resource");

        let mut lrp = LocalResourcePool::default();
        lrp.add_resource(resource).unwrap();

        let resource_from_lrp = lrp.get_resource_by_name(String::from("Test")).unwrap().id;
        let zero_allocations = lrp.get_resource_existing_allocations(&resource_from_lrp);

        assert_eq!(zero_allocations.len(), 0);

        let ar1 = AllocationRequest::new(
            resource_from_lrp,
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            0.2,
            TimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            )
            .unwrap(),
        );

        let ar2 = AllocationRequest::new(
            resource_from_lrp,
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            0.4,
            TimeWindow::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            )
            .unwrap(),
        );

        lrp.allocate(ar1, &project_calendar)
            .expect("Allocation not completed");

        let one_allocations = lrp.get_resource_existing_allocations(&resource_from_lrp);
        assert_eq!(one_allocations.len(), 1);

        lrp.allocate(ar2, &project_calendar)
            .expect("Allocation is not completed");

        let two_allocations = lrp.get_resource_existing_allocations(&resource_from_lrp);
        assert_eq!(two_allocations.len(), 2);
    }
}
