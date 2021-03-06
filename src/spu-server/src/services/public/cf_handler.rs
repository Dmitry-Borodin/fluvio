use log::debug;
use log::trace;
use log::warn;
use log::error;

use futures::stream::StreamExt;
use tokio::select;

use flv_future_aio::sync::broadcast::RecvError;
use kf_socket::KfStream;
use kf_socket::KfSink;
use kf_socket::KfSocketError;
use kf_protocol::api::RequestMessage;
use kf_protocol::api::RequestHeader;
use kf_protocol::api::Offset;
use kf_protocol::api::Isolation;
use flv_metadata::partition::ReplicaKey;
use kf_protocol::fs::FilePartitionResponse;
use spu_api::fetch::FileFlvContinuousFetchRequest;
use spu_api::fetch::FlvContinuousFetchResponse;
use spu_api::SpuApiKey;
use spu_api::PublicRequest;

use crate::core::DefaultSharedGlobalContext;


/// continuous fetch handler
/// while client is active, it continuously send back new records
pub struct CfHandler {
    ctx: DefaultSharedGlobalContext,
    replica: ReplicaKey,
    isolation: Isolation,
    header: RequestHeader,
    kf_sink: KfSink,
}

impl CfHandler {
    /// handle fluvio continuous fetch request
    pub async fn handle_continuous_fetch_request(
        request: RequestMessage<FileFlvContinuousFetchRequest>,
        ctx: DefaultSharedGlobalContext,
        kf_sink: KfSink,
        kf_stream: KfStream,
    ) -> Result<(), KfSocketError> {
        // first get receiver to offset update channel to we don't missed events

        let (header, msg) = request.get_header_request();

        let current_offset = msg.fetch_offset;
        let isolation = msg.isolation;
        let replica = ReplicaKey::new(msg.topic, msg.partition);

        debug!(
            "conn: {}, start continuous fetch replica: {} offset: {}",
            kf_sink.id(),
            replica,
            current_offset
        );

        let mut handler = Self {
            ctx,
            isolation,
            replica,
            header,
            kf_sink,
        };

        handler.process(current_offset, kf_stream).await
    }

    async fn process(
        &mut self,
        starting_offset: Offset,
        mut kf_stream: KfStream,
    ) -> Result<(), KfSocketError> {
        let mut current_offset =
            if let Some(offset) = self.send_back_records(starting_offset).await? {
                offset
            } else {
                debug!(
                    "conn: {}, no records, finishing processing",
                    self.kf_sink.id()
                );
                return Ok(());
            };

        let mut receiver = self.ctx.offset_channel().receiver();
        //pin_mut!(receiver);

        let mut api_stream = kf_stream.api_stream::<PublicRequest, SpuApiKey>();

        let mut counter: i32  = 0;
        loop {
            counter += 1;
            debug!("conn: {}, waiting event, counter: {}", self.kf_sink.id(),counter);

       
            
            select! {
                offset_event_res = receiver.recv() => {

                    match offset_event_res {
                        Ok(offset_event) => {

                            debug!("conn: {}, received offset event connection: {:#?}", self.kf_sink.id(),offset_event);
                            if offset_event.replica_id == self.replica {
                                // depends on isolation, we need to keep track different offset
                                let update_offset = match self.isolation {
                                    Isolation::ReadCommitted => offset_event.hw,
                                    Isolation::ReadUncommitted => offset_event.leo
                                };
                                debug!("conn: {}, update offset: {}",self.kf_sink.id(),update_offset);
                                if update_offset != current_offset {
                                    debug!("conn: {}, updated offset replica: {} offset: {} diff from prev: {}",self.kf_sink.id(), self.replica,update_offset,current_offset);
                                    if let Some(offset) = self.send_back_records(current_offset).await? {
                                        debug!("conn: {}, replica: {} read offset: {}",self.kf_sink.id(), self.replica,offset);
                                        current_offset = offset;
                                    } else {
                                        debug!("conn: {}, no more replica: {} records can be read", self.kf_sink.id(),self.replica);
                                        break;
                                    }
                                } else {
                                    debug!("conn: {}, no changed in offset: {} offset: {} ignoring",self.kf_sink.id(), self.replica,update_offset);
                                }
                            } else {
                                debug!("conn: {}, ignoring event because replica does not match",self.kf_sink.id());
                            }
                            

                        },
                        Err(err) => {
                            match err {
                                RecvError::Closed => {
                                    warn!("conn: {}, lost connection to leader controller",self.kf_sink.id());
                                },
                                RecvError::Lagged(lag) => {
                                    error!("conn: {}, lagging: {}",self.kf_sink.id(),lag);
                                }
                            }

                        }
                    }

                    
                    
                    
                },

                msg = api_stream.next() => {
                    if let Some(content) = msg {
                        debug!("conn: {}, received msg: {:#?}, continue processing",self.kf_sink.id(),content);
                    } else {
                        debug!("conn: {}, client has disconnected, ending continuous fetching: {}",self.kf_sink.id(),self.replica);
                        break;
                    }

                }
            }
            
        }

        Ok(())
    }

    async fn send_back_records(&mut self, offset: Offset) -> Result<Option<Offset>, KfSocketError> {
        let mut partition_response = FilePartitionResponse::default();
        partition_response.partition_index = self.replica.partition;

        if let Some((hw, leo)) = self
            .ctx
            .leaders_state()
            .read_records(
                &self.replica,
                offset,
                self.isolation.clone(),
                &mut partition_response,
            )
            .await
        {
            debug!(
                "conn: {}, retrieved records replica: {}, from: {} to hw: {}, leo: {}",
                self.kf_sink.id(),
                self.replica,
                offset,
                hw,
                leo
            );
            let response = FlvContinuousFetchResponse {
                topic: self.replica.topic.clone(),
                partition: partition_response,
            };

            let response = RequestMessage::<FileFlvContinuousFetchRequest>::response_with_header(
                &self.header,
                response,
            );
            trace!(
                "conn: {}, sending back file fetch response: {:#?}",
                self.kf_sink.id(),
                response
            );

            self.kf_sink
                .encode_file_slices(&response, self.header.api_version())
                .await?;

            trace!("conn: {}, finish sending fetch response", self.kf_sink.id());

            // get next offset
            let next_offset = match self.isolation {
                Isolation::ReadCommitted => hw,
                Isolation::ReadUncommitted => leo,
            };

            Ok(Some(next_offset))
        } else {
            debug!(
                "conn: {} unable to retrieve records from replica: {}, from: {}",
                self.kf_sink.id(),
                self.replica,
                offset
            );
            // in this case, partition is not founded
            Ok(None)
        }
    }
}
