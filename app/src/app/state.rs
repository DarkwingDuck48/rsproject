use chrono::{NaiveDate, Utc};
use logic::{
    BasicGettersForStructures, DependencyType, ExceptionType, ProjectContainer, RateMeasure,
    SingleProjectContainer,
};
use uuid::Uuid;

use crate::app::{AppTheme, views::View};

pub struct ProjectApp {
    pub(crate) container: SingleProjectContainer,
    pub(crate) selected_tab: View,
    pub(crate) selected_project_id: Option<Uuid>,
    pub(crate) selected_task_id: Option<Uuid>,
    pub(crate) selected_resource_id: Option<Uuid>,
    pub(crate) critical_path: Option<Vec<Uuid>>,
    pub(crate) edit_resource_id: Option<Uuid>,
    pub(crate) edit_task_id: Option<Uuid>,
    pub(crate) show_edit_project_dialog: bool,
    pub(crate) current_theme: AppTheme,

    pub(crate) show_close_project_dialog: bool,
    // Create project dialog
    pub(crate) show_new_project_dialog: bool,
    pub(crate) new_project_name: String,
    pub(crate) new_project_desc: String,
    pub(crate) new_project_start: NaiveDate,
    pub(crate) new_project_end: NaiveDate,
    pub(crate) error_message: Option<String>,

    // Create task dialog
    pub(crate) show_new_task_dialog: bool,
    pub(crate) new_task_name: String,
    pub(crate) new_task_start: NaiveDate,
    pub(crate) new_task_end: NaiveDate,
    pub(crate) new_task_is_summary: bool,
    pub(crate) new_task_dependency_task: Option<Uuid>,
    pub(crate) new_task_dependency_type: Option<DependencyType>,
    pub(crate) selected_task_parent_id: Option<Uuid>,

    // Create resource dialog
    pub(crate) show_new_resource_dialog: bool,
    pub(crate) new_resource_name: String,
    pub(crate) new_resource_rate: String,
    pub(crate) new_resource_measure: RateMeasure,

    // Assign Resource dialog
    pub(crate) show_assign_resource_dialog: bool,
    pub(crate) assign_engagement: String,
    pub(crate) assign_use_full_window: bool,
    pub(crate) assign_custom_start: NaiveDate,
    pub(crate) assign_custom_end: NaiveDate,

    pub(crate) show_unavailable_period_dialog: bool,
    pub(crate) unavailable_start: NaiveDate,
    pub(crate) unavailable_end: NaiveDate,
    pub(crate) unavailable_type: ExceptionType,

    // Gantt chart state
    pub(crate) gantt_day_width: f32,
    pub(crate) gantt_only_critical: bool,
    pub(crate) details_task_id: Option<Uuid>,
    pub(crate) show_task_details_dialog: bool,
}

impl Default for ProjectApp {
    fn default() -> Self {
        let now = Utc::now().date_naive();
        Self {
            container: SingleProjectContainer::new(),
            selected_tab: View::Project,
            new_task_dependency_task: None,
            new_task_dependency_type: None,
            show_close_project_dialog: false,
            critical_path: None,
            show_new_project_dialog: false,
            show_new_task_dialog: false,
            show_new_resource_dialog: false,
            show_assign_resource_dialog: false,
            show_unavailable_period_dialog: false,
            new_project_name: String::new(),
            new_project_desc: String::new(),
            new_project_start: now,
            new_project_end: now,
            new_task_name: String::new(),
            new_task_start: now,
            new_task_end: now,
            error_message: None,
            selected_project_id: None,
            selected_task_id: None,
            selected_resource_id: None,
            assign_engagement: String::from("0.5"),
            new_resource_name: String::new(),
            new_resource_rate: String::from("1000"),
            new_resource_measure: RateMeasure::Hourly,
            unavailable_start: now,
            unavailable_end: now,
            unavailable_type: ExceptionType::Vacation,
            assign_use_full_window: false,
            assign_custom_start: now,
            assign_custom_end: now,
            new_task_is_summary: false,
            selected_task_parent_id: None,
            gantt_day_width: 40.0,
            gantt_only_critical: false,
            details_task_id: None,
            show_task_details_dialog: false,
            edit_resource_id: None,
            edit_task_id: None,

            show_edit_project_dialog: false,
            current_theme: AppTheme::Light,
        }
    }
}

impl ProjectApp {
    pub fn with_container(container: SingleProjectContainer) -> Self {
        let project_id = container
            .list_projects()
            .first()
            .map(|p| *p.get_id())
            .unwrap_or_else(Uuid::new_v4);
        Self {
            container,
            current_theme: AppTheme::Light,
            new_task_dependency_task: None,
            new_task_dependency_type: None,
            show_close_project_dialog: false,
            selected_tab: View::Project,
            selected_project_id: Some(project_id),
            show_new_project_dialog: false,
            new_project_name: String::new(),
            new_project_desc: String::new(),
            new_project_start: Utc::now().date_naive(),
            new_project_end: Utc::now().date_naive(),
            error_message: None,
            show_new_task_dialog: false,
            new_task_name: String::new(),
            new_task_start: Utc::now().date_naive(),
            new_task_end: Utc::now().date_naive(),
            show_new_resource_dialog: false,
            new_resource_name: String::new(),
            new_resource_rate: String::from("1000"),
            new_resource_measure: RateMeasure::Hourly,
            show_assign_resource_dialog: false,
            selected_task_id: None,
            selected_resource_id: None,
            assign_engagement: String::from("0.5"),
            assign_use_full_window: true,
            assign_custom_start: Utc::now().date_naive(),
            assign_custom_end: Utc::now().date_naive(),
            show_unavailable_period_dialog: false,
            unavailable_start: Utc::now().date_naive(),
            unavailable_end: Utc::now().date_naive(),
            unavailable_type: ExceptionType::Vacation,
            critical_path: None,
            new_task_is_summary: false,
            selected_task_parent_id: None,
            gantt_day_width: 40.0,
            gantt_only_critical: false,
            details_task_id: None,
            show_task_details_dialog: false,
            edit_resource_id: None,
            edit_task_id: None,

            show_edit_project_dialog: false,
        }
    }
}
