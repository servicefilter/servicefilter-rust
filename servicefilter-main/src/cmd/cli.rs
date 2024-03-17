use crate::error::ServicefilterError;
use servicefilter_core::Result;
use structopt::StructOpt;
use tokio::{fs, signal::unix::{signal, SignalKind}, sync::mpsc::Sender};

/// https://www.nginx.com/resources/wiki/start/topics/tutorials/commandline/
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "servicefilter", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = "servicefilter")]
pub struct Cli {
    #[structopt(name = "Don’t run, just test the configuration file. Servicefilter checks configuration for correct syntax and then try to open files referred in configuration.", short = "t")]
    pub test_config_file: Option<String>,
    
    #[structopt(name = "Specify which configuration file servicefilter should use instead of the default.", short = "c")]
    pub config_file: Option<String>,
    
    #[structopt(name = "Set app id.", short = "id")]
    pub app_id: Option<String>,

    #[structopt(name = "Set remote config args ", short = "r")]
    pub remote_args: Option<String>,

    // #[structopt(name = "Send signal to a master process: stop, quit, reopen, reload.", short = "s")]
    // signal: Option<String>,
}

impl Cli {
    pub async fn check(&self) -> Result<()> {
        if None == self.config_file && None == self.remote_args {
            return Err(ServicefilterError::CliParseError(String::from("config_file and remote_coinfig_args cann't empty at the same time!")).into());
        }

        if None != self.remote_args && None != self.app_id {
            return Err(ServicefilterError::CliParseError(String::from("use remote_args, node_id must not be set!")).into());
        }
 

        if let Some(file_path) = &self.config_file {
            let file_exists = fs::try_exists(file_path).await;
            if let Ok(exists_flag) = file_exists {
                if !exists_flag {
                    return Err(ServicefilterError::CliParseError(format!("{} config file not exists", file_path).into()).into());
                }
            } else {
                return Err(ServicefilterError::CliParseError(format!("{} config file access fail！", file_path)).into());
            }
        }

        Ok(())
    }

    pub fn listen_config_change(&self, config_change_tx: Sender<()>,) {
        if let Some(_) = &self.config_file {
            tokio::spawn(async move{
                let sig_result = signal(SignalKind::hangup());
                if let Ok(mut sig) = sig_result {
                    loop {
                        sig.recv().await;
                        let send_result = config_change_tx.send(()).await;
                        if let Err(e) = send_result {
                            tracing::warn!("config change error: {:?}", e);
                        }
                    }                
                }
            });
        }
    } 
}