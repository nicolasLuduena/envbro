use anyhow::Result;

pub mod iroh;

/// Trait for storage backends.
/// Allows swapping the implementation (e.g. for testing or different backends).
#[async_trait::async_trait]
pub trait Store {
    /// store a value for a specific project environment variable
    async fn set_env_var(
        &mut self,
        project: &str,
        env: &str,
        key: &str,
        value: &[u8],
    ) -> Result<()>;

    /// Store multiple environment variables in a single batch operation
    async fn set_env_vars(
        &mut self,
        project: &str,
        env: &str,
        entries: Vec<(String, Vec<u8>)>,
    ) -> Result<()>;

    /// list all environment variables for a project environment
    async fn list_env_vars(&mut self, project: &str, env: &str) -> Result<Vec<(String, Vec<u8>)>>;
}
