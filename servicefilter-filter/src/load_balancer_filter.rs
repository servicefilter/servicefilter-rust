use std::{collections::HashMap, sync::{Arc, RwLock}};

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use servicefilter_core::{filter::{ServiceInstance, ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}};


pub struct LoadBalancerFilter {

    service_base_load_info: Arc<ServiceConfig>,
    filter_config: FilterConfig,
    client_factory: LoadBalancerClientFactory,
}

impl LoadBalancerFilter {
    pub fn new(
        service_base_load_info: Arc<ServiceConfig>,
        filter_config: FilterConfig,
        ) -> Self {
        let args = filter_config.args.clone();
        Self{
            service_base_load_info,
            filter_config,
            client_factory: LoadBalancerClientFactory::new(args),
        }
    }
}

#[async_trait]
impl ServicefilterFilter for LoadBalancerFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {

        let load_balancer = self.client_factory.choose_load_balancer(exchange).unwrap();

        let instance = load_balancer.choose(exchange);
        if let Some(instance) = instance {
            exchange.set_service_instance(instance);
        }

        tracing::info!("LoadBalancerFilter id: {}, target service name: {:?}",self.filter_config.filter_id, &exchange.get_target_service_name());
        chain.do_chian(exchange).await;
    }
}

trait ServiceInstanceListSupplier : Send + Sync {
    
    fn service_name(&self) -> &String;

    fn fetch_instances(&self) -> Vec<Box<dyn ServiceInstance>>;

}

struct StaticServiceInstanceListSupplier {
    service_name: String,
    channels: ServiceChannelsProperties,
}

impl StaticServiceInstanceListSupplier {
    fn new(service_name: String,
        channels: ServiceChannelsProperties,) -> Self {
        Self { service_name: service_name, channels: channels }
    }


}

impl ServiceInstanceListSupplier for StaticServiceInstanceListSupplier {
    fn service_name(&self) -> &String {
        &self.service_name
    }

    fn fetch_instances(&self) -> Vec<Box<dyn ServiceInstance>> {
        match self.channels.channels.get(&self.service_name) {
            Some(channel) => {
                let instance = StaticServiceInstance {
                    service_id: channel.instance_id.clone(),
                    service_name: self.service_name.clone(),
                    service_protocol: channel.protocol.clone(),
                    address: channel.address.clone(),
                    attributes: channel.attributes.clone(),
                };
                return vec![Box::new(instance)];
            }
            None => {
                return vec![];
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceChannelsProperties {
    channels: HashMap<String, ServiceChannelProperties>,

}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceChannelProperties {
    instance_id: String,
    protocol: String,
    address: String,
    enable_keep_alive: bool,
    negotiation_type: String,
    attributes: HashMap<String, String>
}



trait LoadBalancer: Send + Sync {
    fn choose(&self, exchange: &mut dyn ServicefilterExchange) -> Option<Box<dyn ServiceInstance>>;
}

struct RoundRobinLoadBalancer {
    position: i32,
    service_name: String,
    supplier: Box<dyn ServiceInstanceListSupplier>,
}

impl RoundRobinLoadBalancer {
    pub fn new(service_name: String,supplier: Box<dyn ServiceInstanceListSupplier>,) -> Self {
        Self {
            position: 0,
            service_name: service_name,
            supplier: supplier,
        }
    }
}

impl LoadBalancer for RoundRobinLoadBalancer {

    fn choose(&self, exchange: &mut dyn ServicefilterExchange) -> Option<Box<dyn ServiceInstance>> {
        let mut instances = self.supplier.fetch_instances();
        if let Some(instance) = instances.pop() {
            return Some(instance);
        }
        return None;
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


struct LoadBalancerClientFactory {
    contexts: RwLock<HashMap<String, Arc<Box<dyn LoadBalancer>>>>,
    args: HashMap<String, String>,
}

impl LoadBalancerClientFactory {
    pub fn new(args: HashMap<String, String>,) -> Self {
        Self { 
            contexts: RwLock::new(HashMap::new()),
            args: args,
        }
    }

    fn choose_load_balancer(&self, exchange: &mut dyn ServicefilterExchange) -> Option<Arc<Box<dyn LoadBalancer>>> {
        
        let target_service_name = exchange.get_target_service_name();

        let contexts_read_lock = self.contexts.read().unwrap();
        let load_balancer_op = match contexts_read_lock.get(target_service_name) {
            Some(load_balancer) => {
                let load_balancer = load_balancer.clone();
                drop(contexts_read_lock);

                Some(load_balancer)
            }
            None => {
                
                drop(contexts_read_lock);

                let mut contexts_write_lock = self.contexts.write().unwrap();

                // TODO new by config

                let channels: ServiceChannelsProperties = serde_yaml::from_str(self.args.get("services").unwrap()).unwrap();
                let supplier = StaticServiceInstanceListSupplier::new(String::from(target_service_name), channels);

                contexts_write_lock.insert(String::from(target_service_name), Arc::new(Box::new(RoundRobinLoadBalancer::new(String::from(target_service_name), Box::new(supplier)))));
                let load_balancer = contexts_write_lock.get(target_service_name).unwrap().clone();
                drop(contexts_write_lock);
                Some(load_balancer)
            }
        };

        return load_balancer_op;

    }
}
