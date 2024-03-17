use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use servicefilter_core::{filter::{ServiceInstance, ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}};

pub struct InstanceSetFilter {
    service_config_base: Arc<ServiceConfig>,
    filter_config: FilterConfig
}

impl InstanceSetFilter {
    pub fn new(
        service_config_base: Arc<ServiceConfig>,
        filter_config: FilterConfig,) -> Self {
        Self{
            service_config_base,
            filter_config,
        }
    }
}

#[async_trait]
impl ServicefilterFilter for InstanceSetFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        let listen = &self.service_config_base.listen;
        if let Some(listen) = listen{
            let instance = StaticServiceInstance {
                service_id: self.service_config_base.service_id.clone(),
                service_name: self.service_config_base.service_name.clone(),
                service_protocol: listen.protocol.clone(),
                address: listen.address.clone(),
                attributes: HashMap::new(),
            };
            exchange.set_service_instance(Box::new(instance));
        }
        

        chain.do_chian(exchange).await;
    }
}

#[derive(Debug, Clone)]
struct StaticServiceInstance {
    service_id: String,
    service_name: String,
    service_protocol: String,
    address: String,
    attributes: HashMap<String, String>,

}

impl ServiceInstance for StaticServiceInstance {

    fn get_service_id(&self) -> &String{
        return &self.service_id;
    }

    fn get_service_name(&self) -> &String{
        return &self.service_name;
    }

    fn get_service_protocol(&self) -> &String{
        return &self.service_protocol;
    }

    fn get_address(&self) -> &String{
        return &self.address;
    }

    fn get_attributes(&self) -> &HashMap<String, String>{
        return &self.attributes;
    }
}
