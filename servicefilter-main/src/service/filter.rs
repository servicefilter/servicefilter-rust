use std::{collections::HashMap, mem::replace};

use servicefilter_core::filter::{ServiceInstance, ServicefilterExchange};

pub struct ReqExchange {
    target_service_id: Option<String>,
    target_service_name: String,
    attributes: HashMap<String, String>,
    // TODO change to common req and resp like req{header:{key:value}, body: stream<Bytes>} resp{header:{key:value}, body: Future<Result<stream<Bytes>>, Status>}
    req: Option<http::Request<tonic::body::BoxBody>>,
    resp: Option<tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>>,
    service_instance: Option<Box<dyn ServiceInstance>>,
}

impl ReqExchange {
    pub fn new(target_service_id: Option<String>,
        target_service_name: String,
        attributes: HashMap<String, String>,
        req: Option<http::Request<tonic::body::BoxBody>>,
        resp: Option<tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>>,
        service_instance: Option<Box<dyn ServiceInstance>>,) -> Self {
        Self { target_service_id, target_service_name, attributes, req, resp, service_instance }
    }
}

impl ServicefilterExchange for ReqExchange {

    fn get_target_service_id(&self) -> &Option<String> {
        return &self.target_service_id;
    }

    fn get_target_service_name(&self) -> &String {
        return &self.target_service_name;
    }

    fn get_req(&mut self) -> Option<http::Request<tonic::body::BoxBody>> {
        replace(&mut self.req, None)
    }

    fn get_resp(&mut self) -> Option<tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>> {
        replace(&mut self.resp, None)
    }

    fn set_resp(&mut self, resp : tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>,) {
        self.resp = Some(resp);
    }

    fn get_attributes(&self) -> &HashMap<String, String> {
        return &self.attributes;
    }

    fn get_attribute(&self, key: &String) -> Option<&String> {
        return self.attributes.get(key);
    }

    fn put_attribute(&mut self, key: &String, val: &String) {
        self.attributes.insert(String::from(key), String::from(val));
    }

    fn get_service_instance(&self) -> &Option<Box<dyn ServiceInstance>> {
        return &self.service_instance;
    }

    fn set_service_instance(&mut self, service_instance: Box<dyn ServiceInstance>,) {
        self.service_instance = Some(service_instance);
    }
}