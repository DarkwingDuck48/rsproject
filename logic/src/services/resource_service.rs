use crate::base_structures::ProjectContainer;

pub struct ResourceService<'a, C: ProjectContainer> {
    container: &'a mut C,
}

impl<'a, C: ProjectContainer> ResourceService<'a, C> {
    pub fn new(container: &'a mut C) -> Self {
        Self { container }
    }
}
