
pub mod filter;
pub mod service;
pub mod channel;

pub const X_SERVICEFILTER_TARGET_SERVICE_NAME: &str = "x-servicefilter-target-service-name";
pub const X_SERVICEFILTER_TARGET_SERVICE_ID: &str = "x-servicefilter-target-service-id";
pub const X_SERVICEFILTER_TARGET_SERVICE_PROTOCOL: &str = "x-servicefilter-target-service-protocol";

pub type Error = Box<dyn std::error::Error + Sync + Send>;
pub type Result<T> = anyhow::Result<T>;