pub mod gantt;
pub mod project;
pub mod resources;
pub mod task;

#[derive(PartialEq, Clone, Copy)]
pub enum View {
    Project,
    Tasks,
    Resources,
    Gantt,
}
