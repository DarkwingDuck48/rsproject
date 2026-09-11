use chrono::{DateTime, Utc};
use logic::{
    BasicGettersForStructures, Dependency, DependencyType, ResourceAllocation, Task, TaskService,
    TaskStatus, TimeWindow,
};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::state::AppState;

#[derive(Serialize, Clone)]
pub struct TaskDependencyInfo {
    pub depends_on: Uuid,
    pub dependency_type: DependencyType,
    pub lag_days: Option<i64>,
}

impl TaskDependencyInfo {
    pub fn from_dependency(dependency: Dependency) -> Self {
        Self {
            depends_on: dependency.depends_on,
            dependency_type: dependency.dependency_type,
            lag_days: dependency.lag.map(|td| td.num_days()),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct TaskAllocationInfo {
    pub allocation_id: Uuid,
    pub resource_id: Uuid,
    pub engagement_rate: f64,
    pub time_window: TimeWindow,
}

impl TaskAllocationInfo {
    pub fn from_allocation(allocation: &ResourceAllocation) -> Self {
        Self {
            allocation_id: allocation.get_id(),
            resource_id: *allocation.get_resource_id(),
            engagement_rate: *allocation.get_engagement_rate(),
            time_window: *allocation.get_time_window(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct TaskUpdateDto {
    pub name: Option<String>,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
    pub status: Option<TaskStatus>,
    // Это нужно, чтобы различать ситуации, когда мы не поменяли parent_id
    // и когда мы хотим его выставить как None
    pub parent_id: Option<Option<Uuid>>,
}

#[derive(Serialize, Clone)]
pub struct TaskInfo {
    id: Uuid,
    name: String,
    date_start: DateTime<Utc>,
    date_end: DateTime<Utc>,
    duration_days: i64,
    status: TaskStatus,
    is_summary: bool,
    parent_id: Option<Uuid>,
    cost: f64,
    dependencies: Vec<TaskDependencyInfo>,
    allocations_count: usize,
}

impl TaskInfo {
    pub fn from_task(task: &Task, cost: f64) -> Self {
        Self {
            id: *task.get_id(),
            name: task.name.clone(),
            date_start: task.date_start,
            date_end: task.date_end,
            duration_days: task.get_duration().num_days(),
            status: task.get_status().clone(),
            is_summary: task.is_summary,
            parent_id: task.parent_id,
            cost,
            dependencies: task
                .get_dependencies()
                .iter()
                .map(|dep| TaskDependencyInfo::from_dependency(*dep))
                .collect(),
            allocations_count: task.get_resource_allocations().len(),
        }
    }

    pub fn from_state(
        state: tauri::State<'_, AppState>,
        project_id: Uuid,
        task_id: Uuid,
    ) -> Result<Self, String> {
        let mut container = state.container();
        let task_service = TaskService::new(&mut *container);
        let task = task_service
            .get_task_by_id(&project_id, &task_id)
            .ok_or(format!("Не найдена задача с ID {}", task_id))?;
        let task_cost = task_service
            .calculate_task_cost(&project_id, &task_id)
            .unwrap_or(0.0);
        Ok(Self::from_task(task, task_cost))
    }
}

#[derive(Serialize, Clone)]
pub struct TaskDetailInfo {
    #[serde(flatten)]
    pub task: TaskInfo,
    pub allocations: Vec<TaskAllocationInfo>,
}

#[derive(Serialize, Clone)]
pub struct TaskTreeNode {
    pub task: TaskInfo,
    pub children: Vec<TaskTreeNode>,
}
