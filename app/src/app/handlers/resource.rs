use logic::{
    BasicGettersForStructures, ExceptionPeriod, ProjectContainer, ResourceService, TaskService,
    TimeWindow,
};
use uuid::Uuid;

use crate::ProjectApp;

impl ProjectApp {
    pub fn open_edit_resource_dialog(&mut self, resource_id: Uuid) {
        if let Some(resource) = self.container.resource_pool().get_resource(&resource_id) {
            self.new_resource_name = resource.name.clone();
            self.new_resource_rate = resource.rate.to_string();
            self.new_resource_measure = resource.rate_measure.clone();
            self.edit_resource_id = Some(resource_id);
            self.show_new_resource_dialog = true;
        }
    }

    pub fn create_resource(&mut self) -> anyhow::Result<()> {
        let rate: f64 = self.new_resource_rate.parse()?;
        let mut resource_service = ResourceService::new(&mut self.container);
        if let Some(id) = self.edit_resource_id {
            // Обновление
            resource_service.update_resource(
                id,
                Some(self.new_resource_name.clone()),
                Some(rate),
                Some(self.new_resource_measure.clone()),
            )?;
        } else {
            // Создание
            let resource = resource_service.create_resource(
                self.new_resource_name.clone(),
                rate,
                self.new_resource_measure.clone(),
            )?;
            resource_service.add_resource(resource)?;
        }
        self.new_resource_name.clear();
        self.new_resource_rate = String::from("1000");
        self.edit_resource_id = None;
        Ok(())
    }

    pub fn add_unavailable_period(&mut self) -> anyhow::Result<()> {
        let resource_id = self
            .selected_resource_id
            .ok_or_else(|| anyhow::anyhow!("Не выбран ресурс"))?;
        let period = TimeWindow::new(
            self.unavailable_start
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            self.unavailable_end.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )?;
        let exception_period = ExceptionPeriod {
            period,
            exception_type: self.unavailable_type.clone(),
        };
        let mut resource_service = ResourceService::new(&mut self.container);
        resource_service.add_unavailable_period(resource_id, exception_period)?;
        Ok(())
    }

    pub fn assing_resource(&mut self) -> anyhow::Result<()> {
        let binding = self.container.list_projects();
        let project = binding
            .first()
            .ok_or_else(|| anyhow::anyhow!("Не выбран проект"))?;
        let project_id = *project.get_id();

        let task_id = self
            .selected_task_id
            .ok_or_else(|| anyhow::anyhow!("Не выбрана задача"))?;
        let resource_id = self
            .selected_resource_id
            .ok_or_else(|| anyhow::anyhow!("Не выбран ресурс"))?;
        let engagement: f64 = self.assign_engagement.parse()?;
        if !(0.0..=1.0).contains(&engagement) {
            anyhow::bail!("Занятость должна быть между 0.0 и 1.0");
        }
        let time_window = if self.assign_use_full_window {
            None
        } else {
            let start = self
                .assign_custom_start
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let end = self
                .assign_custom_end
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            Some(TimeWindow::new(start, end)?)
        };
        let mut task_service = TaskService::new(&mut self.container);
        task_service.allocate_resource(
            project_id,
            task_id,
            resource_id,
            engagement,
            time_window,
        )?;
        Ok(())
    }
}
