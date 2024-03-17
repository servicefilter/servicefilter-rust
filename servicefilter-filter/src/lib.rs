use std::{default, sync::Arc};

use empty_response_filter::EmptyResponseFilter;
use filter_req_test_filter::FilterReqTestFilter;
use forward_switch_filter::ForwardSwitchFilter;
use grpc_routing_filter::GrpcRoutingFilter;
use instance_set_filter::InstanceSetFilter;
use load_balancer_filter::LoadBalancerFilter;
use log_filter::LogFilter;
use mock_local_filter::MockLocalFilter;
use servicefilter_core::{filter::ServicefilterFilter, service::{FilterConfig, ServiceConfig}};
use timeout_filter::TimeoutFilter;

mod empty_response_filter;
mod filter_req_test_filter;
mod forward_switch_filter;
mod grpc_routing_filter;
mod instance_set_filter;
mod load_balancer_filter;
mod log_filter;
mod mock_local_filter;
mod timeout_filter;


pub async fn load_req_filter(filter_plugin_name: String, service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig,) -> Option<Box<dyn ServicefilterFilter>> {
    match filter_plugin_name.as_str() {
        "forward_switch_filter" => {
            return Some(Box::new(ForwardSwitchFilter::new(service_config_base, filter_config)));
        },
        "grpc_routing_filter" => {
            return Some(Box::new(GrpcRoutingFilter::new(service_config_base, filter_config)));
        },
        "instance_set_filter" => {
            return Some(Box::new(InstanceSetFilter::new(service_config_base, filter_config)));
        },
        "load_balancer_filter" => {
            return Some(Box::new(LoadBalancerFilter::new(service_config_base, filter_config)));
        },
        "log_filter" => {
            return Some(Box::new(LogFilter::new(service_config_base, filter_config)));
        },
        "timeout_filter" => {
            return Some(Box::new(TimeoutFilter::new()));
        },
        _ => {return None;},
    }
} 

pub async fn load_filter(filter_plugin_name: String, service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig, channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,) -> Option<Box<dyn ServicefilterFilter>> {

    match filter_plugin_name.as_str() {
        "empty_response_filter" => {
            return Some(Box::new(EmptyResponseFilter::new(service_config_base, filter_config)));
        },
        "filter_req_test_filter" => {
            return Some(Box::new(FilterReqTestFilter::new(service_config_base, filter_config, channel_gen)));
        },
        "forward_switch_filter" => {
            return Some(Box::new(ForwardSwitchFilter::new(service_config_base, filter_config)));
        },
        "grpc_routing_filter" => {
            return Some(Box::new(GrpcRoutingFilter::new(service_config_base, filter_config)));
        },
        "instance_set_filter" => {
            return Some(Box::new(InstanceSetFilter::new(service_config_base, filter_config)));
        },
        "load_balancer_filter" => {
            return Some(Box::new(LoadBalancerFilter::new(service_config_base, filter_config)));
        },
        "log_filter" => {
            return Some(Box::new(LogFilter::new(service_config_base, filter_config)));
        },
        "mock_local_filter" => {
            return Some(Box::new(MockLocalFilter::new(service_config_base, filter_config, channel_gen)));
        },
        "timeout_filter" => {
            return Some(Box::new(TimeoutFilter::new()));
        },
        _ => {return None;},
    }

}