use std::sync::Arc;

use servicefilter_lib_mock_server_filter::ServicefilterLibMockServerFilter;
use servicefilter_core::{filter::ServicefilterFilter, service::{FilterConfig, ServiceConfig}};

mod servicefilter_lib_mock_server_filter;

#[no_mangle]
pub extern "C" fn load_filter(filter_plugin_name: String, service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig, channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,) -> Option<*mut dyn ServicefilterFilter> {

    match filter_plugin_name.as_str() {
        "servicefilter_lib_mock_server" => {
            return Some(Box::into_raw(Box::new(ServicefilterLibMockServerFilter::new(service_config_base, filter_config, channel_gen))));
        },
        _ => {return None;},
    }
}