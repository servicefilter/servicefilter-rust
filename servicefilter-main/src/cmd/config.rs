use std::collections::HashMap;

use servicefilter_core::Result;
use tokio::fs::read_to_string;
use serde::{Serialize, Deserialize};

use crate::error::ServicefilterError;

use super::cli::Cli;

#[derive(Debug, Clone)]
pub struct ServicefilterAppConfig {
    pub app_id: String,
    pub lib_load_path: String,
    pub control_listen: ServicefilterListenConfig,
    pub services: Vec<ServicefilterServiceConfig>,    
}

#[derive(Debug, Clone)]
pub struct ServicefilterServiceConfig {
    pub service_id: String,
    pub service_name: String,
    pub alias_names: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub listen: ServicefilterListenConfig,
    pub local_listen: Option<ServicefilterListenConfig>,
    pub service_listen: Option<ServicefilterListenConfig>,
    pub filter: ServicefilterFilterConfig,
}


#[derive(Debug, Clone)]
pub struct ServicefilterListenConfig {
    pub plugin_name: String,
    pub protocol: String,
    pub address: String,
    pub args: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ServicefilterFilterConfig {
    pub channel: Vec<ServicefilterFilterDefineConfig>,
    pub prerouting: Vec<ServicefilterFilterDefineConfig>,
    pub input: Vec<ServicefilterFilterDefineConfig>,
    pub local: Vec<ServicefilterFilterDefineConfig>,
    pub output: Vec<ServicefilterFilterDefineConfig>,
    pub forward: Vec<ServicefilterFilterDefineConfig>,
    pub postrouting: Vec<ServicefilterFilterDefineConfig>,
}

#[derive(Debug, Clone)]
pub struct ServicefilterFilterDefineConfig {
    pub filter_id: String,
    pub filter_name: String,
    pub plugin_name: String,
    pub args: HashMap<String, String>,
}


impl ServicefilterAppConfig {

    pub async fn from_cli(cli : &Cli) -> Result<ServicefilterAppConfig> {
        if let Some(file_path) = &cli.config_file {
            let config_str = read_to_string(file_path).await?;
            let config_file: ServicefilterAppConfigFile = serde_yaml::from_str(config_str.as_str())?;

            let config = file_to_config(config_file);
            return Ok(config);
        }

        // if let Some(remote_args) = &cli.remote_args {
        //     if let Some(server_id) = &cli.server_id {
        //         // TODO design plugin load
        //     }
        // }

        Err(ServicefilterError::CliParseError("parse config error, please check command line args".into()).into())
    }
}

fn file_to_config(config_file: ServicefilterAppConfigFile) -> ServicefilterAppConfig {
    let control_liten_file: ServicefilterListenConfigFile = config_file.control_listen;
    let control_listen = file_listen_to_listen(control_liten_file);

    let file_services = config_file.services;
    let mut services = Vec::new();
    for file_service in file_services {
        let service_config = ServicefilterServiceConfig{
            service_id: file_service.service_id,
            service_name: file_service.service_name,
            alias_names: file_service.alias_names,
            attributes: file_service.attributes,
            listen: file_listen_to_listen(file_service.listen),
            local_listen: file_listen_to_option_listen(file_service.local_listen),
            service_listen: file_listen_to_option_listen(file_service.service_listen),
            filter: file_filter_to_filter(file_service.filter),
            // server_discover: file_service_discover_to_service_discover(file_service.server_discover),
        };
        services.push(service_config);
    }

    return ServicefilterAppConfig{app_id: config_file.app_id, lib_load_path: config_file.lib_load_path, control_listen, services};
}

fn file_listen_to_listen(file_listen: ServicefilterListenConfigFile) -> ServicefilterListenConfig {

    let listen = ServicefilterListenConfig{
        plugin_name: file_listen.plugin_name,
        protocol: file_listen.protocol,
        address: file_listen.address,
        args: file_listen.args,
    };
    return  listen;

}

fn file_listen_to_option_listen(file_listen_op: Option<ServicefilterListenConfigFile>) -> Option<ServicefilterListenConfig> {
    if let Some(file_listen) = file_listen_op {
        return Some(file_listen_to_listen(file_listen));
    }
    return None;
}

fn file_filter_to_filter(file_filter: ServicefilterFilterConfigFile) -> ServicefilterFilterConfig {
    return ServicefilterFilterConfig {
        channel: file_filter_defines_to_filter_defines(file_filter.channel),
        prerouting: file_filter_defines_to_filter_defines(file_filter.prerouting),
        input: file_filter_defines_to_filter_defines(file_filter.input),
        local: file_filter_defines_to_filter_defines(file_filter.local),
        output: file_filter_defines_to_filter_defines(file_filter.output),
        forward: file_filter_defines_to_filter_defines(file_filter.forward),
        postrouting: file_filter_defines_to_filter_defines(file_filter.postrouting),
    };
}

fn file_filter_defines_to_filter_defines(file_filter_defines: Vec<ServicefilterFilterDefineConfigFile>) -> Vec<ServicefilterFilterDefineConfig> {

    let mut filter_defines = Vec::new();
    for file_filter_define in file_filter_defines  {
        let filter_define = ServicefilterFilterDefineConfig {
            filter_id: file_filter_define.filter_id,
            filter_name: file_filter_define.filter_name,
            plugin_name: file_filter_define.plugin_name,
            args: file_filter_define.args,
        };

        filter_defines.push(filter_define);
    }

    return filter_defines;
}



#[derive(Debug, Serialize, Deserialize)]
pub struct ServicefilterAppConfigFile {
    app_id: String,
    lib_load_path: String,
    control_listen: ServicefilterListenConfigFile,
    services: Vec<ServicefilterServiceConfigFile>,    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServicefilterServiceConfigFile {
    service_id: String,
    service_name: String,
    alias_names: Vec<String>,
    attributes: HashMap<String, String>,
    listen: ServicefilterListenConfigFile,
    local_listen: Option<ServicefilterListenConfigFile>,
    service_listen: Option<ServicefilterListenConfigFile>,
    // channel_filter: Vec<ServicefilterFilterDefineConfigFile>,
    filter: ServicefilterFilterConfigFile,
    // server_discover: ServicefilterServiceDiscoveryConfigFile,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ServicefilterListenConfigFile {
    plugin_name: String,
    protocol: String,
    address: String,
    args: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServicefilterFilterConfigFile {
    channel: Vec<ServicefilterFilterDefineConfigFile>,
    prerouting: Vec<ServicefilterFilterDefineConfigFile>,
    input: Vec<ServicefilterFilterDefineConfigFile>,
    local: Vec<ServicefilterFilterDefineConfigFile>,
    output: Vec<ServicefilterFilterDefineConfigFile>,
    forward: Vec<ServicefilterFilterDefineConfigFile>,
    postrouting: Vec<ServicefilterFilterDefineConfigFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServicefilterFilterDefineConfigFile {
    filter_id: String,
    filter_name: String,
    plugin_name: String,
    args: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct  ServicefilterServiceDiscoveryConfigFile {
    plugin_name: String,
    args: HashMap<String, String>,
}
