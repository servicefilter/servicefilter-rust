use thiserror::Error;


#[derive(Error, Debug)]
pub enum ServicefilterError {

    #[error("Cli_Parse_Error: {0}")]
    CliParseError(String),
}