//!
//! # Continuous Fetch
//!
//! Stream records to client
//!
use std::fmt::Debug;
use std::io::Error as IoError;
use std::marker::PhantomData;

use log::trace;

use flv_future_aio::bytes::BytesMut;
use kf_protocol::Version;
use kf_protocol::Encoder;
use kf_protocol::Decoder;
use kf_protocol::api::Request;
use kf_protocol::api::DefaultRecords;
use kf_protocol::api::Isolation;
use kf_protocol::derive::Decode;
use kf_protocol::derive::Encode;
use kf_protocol::fs::StoreValue;
use kf_protocol::fs::KfFileRecordSet;
use kf_protocol::fs::FileWrite;
use kf_protocol::message::fetch::FetchablePartitionResponse;

pub type DefaultFlvContinuousFetchResponse = FlvContinuousFetchResponse<DefaultRecords>;
pub type FileFlvContinuousFetchRequest = FlvContinuousFetchRequest<KfFileRecordSet>;
pub type DefaultFlvContinuousFetchRequest = FlvContinuousFetchRequest<DefaultRecords>;

use crate::SpuApiKey;



/// Fetch records continuously
/// After initial fetch, update to same replica will stream to client
#[derive(Decode, Encode, Default, Debug)]
pub struct FlvContinuousFetchRequest<R> 
    where R: Encoder + Decoder + Default + Debug,
{
    
    pub topic: String,
    pub partition: i32,
    pub fetch_offset: i64,
    pub max_bytes: i32,
    pub isolation: Isolation,
    pub data: PhantomData<R>,
}


impl <R>Request for FlvContinuousFetchRequest<R> 
    where R: Debug + Decoder + Encoder
{
    const API_KEY: u16 = SpuApiKey::FlvContinuousFetch as u16;
    const DEFAULT_API_VERSION: i16 = 10;
    type Response = FlvContinuousFetchResponse<R>;
}


#[derive(Encode, Decode, Default, Debug)]
pub struct FlvContinuousFetchResponse<R>
    where R: Encoder + Decoder + Default + Debug,
{
    pub topic: String,
    pub partition: FetchablePartitionResponse<R>
}


impl FileWrite for FlvContinuousFetchResponse<KfFileRecordSet> {
    fn file_encode(
        &self,
        src: &mut BytesMut,
        data: &mut Vec<StoreValue>,
        version: Version,
    ) -> Result<(), IoError> {

        trace!("file encoding FlvContinuousFetchResponse");
        trace!("topic {}", self.topic);
        self.topic.encode(src, version)?;
        self.partition.file_encode(src, data, version)?;
        Ok(())

    }
}
