use crate::domain::User;

pub trait UserRepo: Sync + Send + 'static {
    fn create(&self, user: User) -> anyhow::Result<()>;
    fn delete(&self, id: String) -> anyhow::Result<()>;
}
