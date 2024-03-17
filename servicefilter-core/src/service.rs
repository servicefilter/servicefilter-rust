use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;

use crate::Result;


#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub service_id: String,
    pub service_name: String,
    pub alias_names: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub listen: Option<ServiceListenConfig>,
}

impl ServiceConfig {
    pub fn new(
        service_id: String,
        service_name: String,
        alias_names: Vec<String>,
        attributes: HashMap<String, String>,
        listen: Option<ServiceListenConfig>,
    ) -> Self {
        Self { service_id, service_name, alias_names, attributes, listen }
    }
}

#[derive(Clone, Debug)]
pub struct ServiceListenConfig {
    pub protocol: String,
    pub address: String,
}

impl ServiceListenConfig {
    pub fn new(
        protocol: String,
        address: String,
    ) -> Self {
        Self { protocol, address }
    }
}

#[derive(Clone, Debug)]
pub struct FilterConfig {
    pub filter_id: String,
    pub filter_name: String,
    pub args: HashMap<String, String>,
}

impl FilterConfig {
    pub fn new(
        filter_id: String,
        filter_name: String,
        args: HashMap<String, String>,) -> Self {
        Self { filter_id, filter_name, args }
    }
}

pub struct ProtocolListenConfig {
    pub protocol: String,
    pub address: String,
    pub args: HashMap<String, String>,
}

impl ProtocolListenConfig {
    pub fn new(
        protocol: String,
        address: String,
        args: HashMap<String, String>,
    ) -> Self {
        Self { protocol, address, args }
    }
}

pub trait ProtocolServer : Sync + Send {

    fn start(&self, service_config: Arc<ServiceConfig>, listen_config: ProtocolListenConfig, gen_chain: Box<dyn ServiceGenChain>) -> Result<Pin<Box<dyn Future<Output=()>+Send>>>;

}

pub trait ProtocolServerGenService : Sync + Send {

    
}

type ServiceConfigModify = Box<dyn Fn(Box<ServiceConfig>) + Send + Sync + 'static>;
type ServiceConfigCheck = Box<dyn Fn(Box<ServiceConfig>) + Send + Sync + 'static>;
type ServiceConfigExtract = Box<dyn Fn() -> HashMap<String, String> + Send + Sync + 'static>;

// TODO mabe only service config
pub struct ServiceConfigOpreate {
    pub config_modify: ServiceConfigModify,
    pub config_extract: ServiceConfigExtract,
    pub config_check: ServiceConfigCheck,
}


type FilterConfigModify = Box<dyn Fn(Box<FilterConfig>) + Send + Sync + 'static>;
type FilterConfigCheck = Box<dyn Fn(Box<FilterConfig>) + Send + Sync + 'static>;
type FilterConfigExtract = Box<dyn Fn() -> HashMap<String, String> + Send + Sync + 'static>;

// TODO mabe only filter config
pub struct FilterConfigOpreate {
    pub config_modify: FilterConfigModify,
    pub config_extract: FilterConfigExtract,
    pub config_check: FilterConfigCheck,
}

#[async_trait]
pub trait ServiceGenChain : Sync + Send {
    async fn gen_chain(&self) -> Box<dyn crate::filter::ServicefilterChain>;
}