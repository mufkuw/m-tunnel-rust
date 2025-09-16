use anyhow::Result;

#[async_trait::async_trait]
pub trait TunnelManagerTrait {
    async fn start(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
}