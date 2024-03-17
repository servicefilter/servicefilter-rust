use std::collections::HashMap;

use async_trait::async_trait;

#[derive(Clone, Copy, Debug)]
pub enum FilterKind {
    PREROUTING,
    INPUT,
    LOCAL,
    OUTPUT,
    FORWARD,
    POSTROUTING,
}

#[async_trait]
pub trait ServicefilterChain : Sync + Send {
    async fn do_chian(&self, exchange : &mut dyn ServicefilterExchange);
}

#[async_trait]
pub trait ServicefilterFilter: Sync + Send {
    
    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain);
}

pub trait ServiceInstance: Sync + Send {
    fn get_service_id(&self) -> &String;
    fn get_service_name(&self) -> &String;
    fn get_service_protocol(&self) -> &String;
    fn get_address(&self) -> &String;
    fn get_attributes(&self) -> &HashMap<String, String>;
}

pub trait ServicefilterExchange : Send {
    fn get_target_service_id(&self) -> &Option<String>;
    fn get_target_service_name(&self) -> &String;

    fn get_req(&mut self) -> Option<hyper::Request<tonic::body::BoxBody>>;

    fn get_resp(&mut self) -> Option<tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>>;
    fn set_resp(&mut self, resp : tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>,);

    fn get_attributes(&self) -> &HashMap<String, String>;
    fn get_attribute(&self, key: &String) -> Option<&String>;
    fn put_attribute(&mut self, key: &String, val: &String);

    fn get_service_instance(&self) -> &Option<Box<dyn ServiceInstance>>;
    fn set_service_instance(&mut self, service_instance: Box<dyn ServiceInstance>,);
}