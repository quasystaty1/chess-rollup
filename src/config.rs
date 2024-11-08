use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    /// The path to penumbra storage db.
    pub db_filepath: PathBuf,
    /// Log level: debug, info, warn, or error
    pub log: String,
    /// The gRPC endpoint
    pub grpc_addr: String,
    /// Forces writing trace data to stdout no matter if connected to a tty or not.
    pub force_stdout: bool,
    /// Writes a human readable format to stdout instead of JSON formatted OTEL trace data.
    pub pretty_print: bool,
}
