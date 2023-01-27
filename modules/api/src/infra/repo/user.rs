use crate::domain::{User, UserRepo};

pub struct UserRepoImpl {}

impl UserRepo for UserRepoImpl {
    fn create(&self, user: User) -> anyhow::Result<()> {
        todo!()
    }

    fn delete(&self, id: String) -> anyhow::Result<()> {
        todo!()
    }
}
