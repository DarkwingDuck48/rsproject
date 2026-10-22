use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    RateMeasure,
    base_structures::{
        project_calendar::ProjectCalendar, resource::Resource, time_window::TimeWindow,
        traits::ResourcePool,
    },
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
}

// Объект для описания назначения одного из ресурсов на задачу
#[derive(Default, Debug, Serialize, Deserialize)]
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

    pub fn get_resource_id(&self) -> &Uuid {
        &self.resource_id
    }

    pub fn get_time_window(&self) -> &TimeWindow {
        &self.time_window
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct LocalResourcePool {
    resources: HashMap<Uuid, Resource>,
    allocations: HashMap<Uuid, ResourceAllocation>,
}

impl LocalResourcePool {
    /// Проверка «ресурс есть в пуле». Нужна только тестам: в рабочем коде та же
    /// проверка идёт через `self.resources.get(...)` в `check_allocation_correct`.
    #[cfg(test)]
    fn check_resource_exists(&self, resource_id: &Uuid) -> bool {
        self.resources.contains_key(resource_id)
    }

    /// Поиск ресурса по имени. Нужен только тестам; если понадобится приложению —
    /// убрать `#[cfg(test)]` и объявить метод в трейте `ResourcePool`.
    #[cfg(test)]
    pub fn get_resource_by_name(&self, find_name: String) -> Option<&Resource> {
        self.resources.values().find(|r| r.name == find_name)
    }

    /// Функция должна проверить, что ресурс можно корректно назначить на
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
    ) -> anyhow::Result<Uuid> {
        match self.check_allocation_correct(&request, calendar) {
            Ok(()) => {
                let allocation = ResourceAllocation::new(request);
                let allocation_id = allocation.get_id();
                self.allocations.insert(allocation.get_id(), allocation);
                Ok(allocation_id)
            }
            Err(e) => Err(e),
        }
    }
    fn get_resources(&self) -> Vec<&Resource> {
        self.resources.values().collect()
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

    fn get_resource_existing_allocations(&self, resource_id: &Uuid) -> Vec<&ResourceAllocation> {
        self.allocations
            .values()
            .filter(|a| &a.resource_id == resource_id)
            .collect()
    }

    fn get_mut_resource_by_uuid(&mut self, resource_id: Uuid) -> Option<&mut Resource> {
        self.resources.get_mut(&resource_id)
    }

    fn get_allocation(&self, allocation_id: &Uuid) -> Option<&ResourceAllocation> {
        self.allocations.get(allocation_id)
    }

    fn get_resource(&self, resource_id: &Uuid) -> Option<&Resource> {
        self.resources.get(resource_id)
    }

    fn calculate_allocation_cost(
        &self,
        allocation_id: &Uuid,
        calendar: &ProjectCalendar,
    ) -> anyhow::Result<f64> {
        let allocation = self
            .allocations
            .get(allocation_id)
            .ok_or_else(|| anyhow::anyhow!("Не найдено назначение"))?;
        let resource = self
            .resources
            .get(&allocation.resource_id)
            .ok_or_else(|| anyhow::anyhow!("Ресурс из назначения не найден!"))?;
        // Определяем длительность работы из назначения
        let hours = allocation.time_window.duration_hours(calendar) as f64;
        // Единая календарь-зависимая конвертация: месячная ставка делится на часы
        // в «среднем месяце» календаря, а не на часы конкретного окна, поэтому
        // стоимость пропорциональна фактическим трудозатратам (см. `RateMeasure::convert`).
        let hourly_rate = resource.get_converted_rate(RateMeasure::Hourly, calendar);
        Ok(hourly_rate * hours * allocation.engagement_rate)
    }

    fn calculate_allocation_time(
        &self,
        allocation_id: &Uuid,
        calendar: &ProjectCalendar,
    ) -> anyhow::Result<f64> {
        let allocation = self
            .allocations
            .get(allocation_id)
            .ok_or_else(|| anyhow::anyhow!("Не найдено назначение"))?;

        if self.get_resource(&allocation.resource_id).is_none() {
            return Err(anyhow::anyhow!("Ресурс из назначения не найден!"));
        }

        // Определяем длительность работы из назначения
        let hours = allocation.time_window.duration_hours(calendar) as f64;
        Ok(hours * allocation.engagement_rate)
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
    fn test_calculate_cost() {
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
                Utc.with_ymd_and_hms(2025, 1, 11, 0, 0, 0).unwrap(),
            )
            .unwrap(),
        );

        let allocation_id = lrp.allocate(allocation_request, &project_calendar).unwrap();
        let cost = lrp
            .calculate_allocation_cost(&allocation_id, &project_calendar)
            .unwrap();
        // cost = hourly_rate * hours * engagement_rate
        // hourly_rate = 1000
        // hours = 8 working days * 8 hours/day = 64 hours
        // engagement_rate = 0.8
        assert_eq!(cost, 1000.0 * 64.0 * 0.8);
    }

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
        let calendar = ProjectCalendar::default(); // 8 ч/день, 22 дня/месяц
        let resource = Resource::new(String::from("Test"), 1000.0, RateMeasure::Hourly)
            .expect("Can't create resource");
        assert_eq!(resource.get_base_rate(), &1000.0);
        assert_eq!(
            resource.get_converted_rate(RateMeasure::Daily, &calendar),
            8000.0 // 1000 × 8 ч/день
        );
        assert_eq!(
            resource.get_converted_rate(RateMeasure::Monthly, &calendar),
            176000.0 // 1000 × 8 ч/день × 22 дня/месяц
        );
    }

    // Конвертация обратима: перевод туда и обратно возвращает исходную ставку.
    // Раньше Hourly→Monthly делал rate×22, а Monthly→Hourly rate÷176 — round-trip
    // не сходился (1000 → 22000 → 125). Теперь оба направления согласованы.
    #[test]
    fn test_rate_conversion_round_trip() {
        let calendar = ProjectCalendar::default(); // 8 ч/день, 22 дня/месяц
        let resource = Resource::new(String::from("Test"), 1000.0, RateMeasure::Hourly)
            .expect("Can't create resource");

        let hourly_daily = resource.get_converted_rate(RateMeasure::Daily, &calendar);
        let daily_hourly = RateMeasure::Daily.convert(RateMeasure::Hourly, hourly_daily, &calendar);
        assert!(
            (daily_hourly - 1000.0).abs() < f64::EPSILON,
            "Hourly→Daily→Hourly: 1000 → {hourly_daily} → {daily_hourly}"
        );

        let hourly_monthly = resource.get_converted_rate(RateMeasure::Monthly, &calendar);
        let monthly_hourly =
            RateMeasure::Monthly.convert(RateMeasure::Hourly, hourly_monthly, &calendar);
        assert!(
            (monthly_hourly - 1000.0).abs() < f64::EPSILON,
            "Hourly→Monthly→Hourly: 1000 → {hourly_monthly} → {monthly_hourly}"
        );

        let daily = 800.0;
        let daily_monthly = RateMeasure::Daily.convert(RateMeasure::Monthly, daily, &calendar);
        let monthly_daily =
            RateMeasure::Monthly.convert(RateMeasure::Daily, daily_monthly, &calendar);
        assert!(
            (monthly_daily - daily).abs() < f64::EPSILON,
            "Daily→Monthly→Daily: {daily} → {daily_monthly} → {monthly_daily}"
        );
    }

    // Число рабочих дней в месяце настраивается в календаре и влияет на конвертацию.
    #[test]
    fn test_rate_conversion_uses_custom_working_days_per_month() {
        // По умолчанию 22 дня: 1000 ₽/ч = 176 000 ₽/мес.
        let default_calendar = ProjectCalendar::default();
        let resource = Resource::new(String::from("Test"), 1000.0, RateMeasure::Hourly)
            .expect("Can't create resource");
        assert_eq!(
            resource.get_converted_rate(RateMeasure::Monthly, &default_calendar),
            1000.0 * 8.0 * 22.0
        );

        // Пользователь задал свой календарь: 20 рабочих дней в месяце → дешевле за месяц.
        let mut custom_calendar = ProjectCalendar::default();
        custom_calendar.working_days_per_month = 20;
        assert_eq!(
            resource.get_converted_rate(RateMeasure::Monthly, &custom_calendar),
            1000.0 * 8.0 * 20.0
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

// Property-based тесты пула ресурсов (задача 6.3).
//
// Инвариант: в любой момент времени сумма engagement всех назначений, покрывающих
// эту точку времени, не должна превышать 1.0 — иначе ресурс перегружен. Пул
// проверяет это при каждом назначении (см. `check_correct_timewindow`), а этот тест
// проверяет свойство на случайных последовательностях назначений.
#[cfg(test)]
mod property_tests {
    use super::*;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use proptest::prelude::*;

    /// Генерирует последовательность запросов на назначение одного ресурса:
    /// (окно в днях от старта, длительность окна, engagement).
    fn allocation_requests_strategy() -> impl Strategy<Value = Vec<(i64, i64, f64)>> {
        prop::collection::vec((0i64..=30, 1i64..=10, 0.1f64..=1.0f64), 1..=8)
    }

    proptest! {
        // Инвариант 6.3: сумма engagement по пересекающимся окнам <= 1.
        // Принятые пулом назначения никогда не перегружают ресурс ни в один момент времени.
        #[test]
        fn overlapping_engagements_do_not_exceed_one(requests in allocation_requests_strategy()) {
            let calendar = ProjectCalendar::default();
            let resource =
                Resource::new("R".into(), 100.0, RateMeasure::Hourly).expect("ресурс создаётся");
            let mut pool = LocalResourcePool::default();
            pool.add_resource(resource.clone()).unwrap();

            let base = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
            let mut accepted: Vec<(TimeWindow, f64)> = Vec::new();

            for (start_day, len_days, engagement) in requests {
                let window = TimeWindow::new(
                    base + Duration::days(start_day),
                    base + Duration::days(start_day + len_days),
                )
                .unwrap();
                let request = AllocationRequest::new(
                    resource.id,
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    engagement,
                    window,
                );
                if pool.allocate(request, &calendar).is_ok() {
                    accepted.push((window, engagement));
                }
            }

            // Проверяем инвариант в характерных точках: центр каждого принятого окна
            // и все его границы. Внутри окна состав покрывающих назначений не меняется,
            // поэтому этих точек достаточно для проверки всего континуума.
            let mut probe_times: Vec<DateTime<Utc>> = Vec::new();
            for (window, _) in &accepted {
                probe_times.push(window.date_start);
                probe_times.push(window.date_end);
                probe_times.push(
                    window.date_start
                        + (window.date_end - window.date_start) / 2,
                );
            }

            let mut sorted = probe_times.clone();
            sorted.sort_unstable();
            sorted.dedup();
            for t in sorted {
                let engagement_sum: f64 = accepted
                    .iter()
                    .filter(|(w, _)| w.contains(&t))
                    .map(|(_, e)| *e)
                    .sum();
                prop_assert!(
                    engagement_sum <= 1.0 + f64::EPSILON * 8.0,
                    "в момент {t:?} сумма engagement = {engagement_sum} > 1"
                );
            }
        }
    }
}
