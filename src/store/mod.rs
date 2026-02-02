use anyhow::Result;

pub mod iroh;

#[async_trait::async_trait]
pub trait Store: Send + Sync {
    /// Stores the encrypted environment data with a filename.
    async fn put(&self, project: &str, env: &str, filename: &str, data: &[u8]) -> Result<()>;

    /// Retrieves the filename and encrypted environment data.
    async fn get(&self, project: &str, env: &str) -> Result<(String, Vec<u8>)>;

    /// Deletes the environment data/reference.
    async fn delete(&self, project: &str, env: &str) -> Result<()>;

    /// Lists all projects.
    async fn list_projects(&self) -> Result<Vec<String>>;

    /// Lists all environments for a given project.
    async fn list_envs(&self, project: &str) -> Result<Vec<String>>;
}
