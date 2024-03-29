mod noop_filter;
mod loading;

use std::sync::Arc;

use noop_filter::NoopFilter;
use servicefilter_core::{filter::ServicefilterFilter, service::{FilterConfig, ProtocolServer, ServiceConfig}};
use servicefilter_protocol_grpc::ProtocolServiceGrpc;


pub struct LoadFactory {
    base_path: String,
}

impl LoadFactory {

    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }

    pub async fn load_filter(&self, filter_plugin_name: String, service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig, channel_gen: Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,) -> Option<Box<dyn ServicefilterFilter>> {

        let filter_op = servicefilter_filter::load_filter(filter_plugin_name.clone(), service_config_base.clone(), filter_config.clone(), channel_gen.clone()).await;
        if let Some(filter) = filter_op {
            return Some(filter);
        }
        let filter_op = loading::load_filter(self.base_path.clone(), String::from(""), filter_plugin_name.clone(), service_config_base, filter_config, channel_gen).await;
        if let Some(filter) = filter_op {
            return Some(filter);
        }
        return Some(Box::new(NoopFilter::new(filter_plugin_name.clone())));
    }

    // TODO remove 
    pub async fn load_req_filter(&self, filter_plugin_name: &String, service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig, ) -> Option<Box<dyn ServicefilterFilter>> {

        let filter_op = servicefilter_filter::load_req_filter(filter_plugin_name.clone(), service_config_base.clone(), filter_config.clone(),).await;
        if let Some(filter) = filter_op {
            return Some(filter);
        }
        
        return Some(Box::new(NoopFilter::new(filter_plugin_name.clone())));
    }

    

    pub fn load_protocol_server(&self, protocol_server_plugin_name: &String) -> Option<Box<dyn ProtocolServer>> {

        return Some(Box::new(ProtocolServiceGrpc::new()));
    }

}
