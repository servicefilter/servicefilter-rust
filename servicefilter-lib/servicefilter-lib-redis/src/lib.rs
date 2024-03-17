use std::sync::Arc;

use redis_routing_filter::RedisRoutingFilter;
use servicefilter_core::{filter::ServicefilterFilter, service::{FilterConfig, ServiceConfig}};

mod redis_routing_filter;

#[no_mangle]
pub extern "C" fn load_filter(filter_plugin_name: String, service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig, channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,) -> Option<Box<dyn ServicefilterFilter>> {

    match filter_plugin_name.as_str() {
        "redis_routing_filter" => {
            return Some(Box::new(RedisRoutingFilter::new()));
        },
        _ => {return None;},
    }
}

// pub async fn load_filter(filter_plugin_name: String, service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig, channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,) -> Option<Box<dyn ServicefilterFilter>> {

//     match filter_plugin_name.as_str() {
//         "redis_routing_filter" => {
//             return Some(Box::new(RedisRoutingFilter::new()));
//         },
//         _ => {return None;},
//     }

// }