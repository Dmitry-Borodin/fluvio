//!
//! # List Custom SPUs CLI
//!
//! CLI tree and processing to list Custom SPUs
//!
use structopt::StructOpt;


use flv_client::profile::ScConfig;

use crate::error::CliError;

use crate::Terminal;
use crate::OutputType;

use crate::spu::helpers::format_spu_response_output;
use crate::spu::helpers::flv_response_to_spu_metadata;



#[derive(Debug)]
pub struct ListCustomSpusConfig {
    pub output: OutputType,
}


#[derive(Debug, StructOpt)]
pub struct ListCustomSpusOpt {
    /// Address of Streaming Controller
    #[structopt(short = "c", long = "sc", value_name = "host:port")]
    sc: Option<String>,

    ///Profile name
    #[structopt(short = "P", long = "profile")]
    pub profile: Option<String>,

    /// Output
    #[structopt(
        short = "O",
        long = "output",
        value_name = "type",
        raw(possible_values = "&OutputType::variants()", case_insensitive = "true")
    )]
    output: Option<OutputType>,
}

impl ListCustomSpusOpt {

        /// Validate cli options and generate config
    fn validate(self) -> Result<(ScConfig, ListCustomSpusConfig), CliError> {

        let target_server = ScConfig::new(self.sc, self.profile)?;

        // transfer config parameters
        let list_custom_spu_cfg = ListCustomSpusConfig {
            output: self.output.unwrap_or(OutputType::default()),
        };

        // return server separately from topic result
        Ok((target_server, list_custom_spu_cfg))
    }

}


/// Process list spus cli request
pub async fn process_list_custom_spus<O>(out: std::sync::Arc<O>,opt: ListCustomSpusOpt) -> Result<(), CliError> 
    where O: Terminal
{

    let (target_server, list_custom_spu_cfg) = opt.validate()?;

    let mut sc = target_server.connect().await?;

    let flv_spus = sc.list_spu(true).await?;

    let sc_spus = flv_response_to_spu_metadata(flv_spus);

    // format and dump to screen
    format_spu_response_output(out,sc_spus, list_custom_spu_cfg.output)?;
    Ok(())
}
