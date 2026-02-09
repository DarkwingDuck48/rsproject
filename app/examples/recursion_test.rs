use std::collections::HashMap;

struct Dependency {
    prev_task: Option<uuid::Uuid>,
    next_task: Option<uuid::Uuid>,
}

#[derive(Default)]
struct Task {
    id: uuid::Uuid,
}

#[derive(Default)]
struct Project {
    tasks: HashMap<uuid::Uuid, Task>,
}

fn main() {
    
}
