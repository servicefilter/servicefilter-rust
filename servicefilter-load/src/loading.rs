use std::{panic, path::Path, sync::Arc};

use libloading::{Library, Symbol};
use servicefilter_core::{filter::ServicefilterFilter, service::{FilterConfig, ServiceConfig}};


pub async fn load_filter(app_sys_load_path: String, extend_path: String, 
    filter_plugin_name: String, service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig, 
    channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,) -> Option<Box<dyn ServicefilterFilter>> {

    let lib_str = "dylib";

    let path = Path::new(&app_sys_load_path);
    if path.exists() && path.is_dir() {
        let dir = tokio::fs::read_dir(path).await;
        if let Ok(mut dir) = dir {
            loop {
                let entry_result = dir.next_entry().await;
                if let Ok(entry_op) = entry_result {
                    if let Some(entry) = entry_op {
                        let path = entry.path();
                        let extension_op = path.extension();
                        if let Some(extension) = extension_op {
                            if extension.to_str().unwrap() != lib_str {
                                continue;
                            }
                        } else {
                            continue;
                        }
                        let metadata_result = entry.metadata().await;
                        if let Ok(metadata) = metadata_result {
                            if metadata.is_file() {
                                let filter = load_lib(
                                    entry.path().into_os_string().into_string().unwrap(), filter_plugin_name.clone(), 
                                    service_config_base.clone(), filter_config.clone(), channel_gen.clone(),);
                                if filter.is_some() {
                                    return filter;
                                }
                            }
                        } else {
                            continue;
                        }

                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    return None;

}


fn load_lib(file: String, filter_plugin_name: String, 
    service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig, 
    channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,) -> Option<Box<dyn ServicefilterFilter>> {
    unsafe {
        if let Ok(lib) = Library::new(file) {
        
            // load factory func
            let load_filter_symbol_result: std::result::Result<Symbol<unsafe extern "C" fn(filter_plugin_name: String, 
                service_config_base: Arc<ServiceConfig>, filter_config: FilterConfig, 
                channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,) -> Option<Box<dyn ServicefilterFilter>>>,  libloading::Error> = lib.get(b"load_filter");
            
            if let Ok(load_filter_symbol) = load_filter_symbol_result {
                let filter = load_filter_symbol(filter_plugin_name, service_config_base, filter_config, channel_gen);
                
                return filter;
            }
        }
    }
    return None;
}