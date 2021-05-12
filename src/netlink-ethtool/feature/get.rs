use futures::{self, future::Either, FutureExt, StreamExt, TryStream};
use netlink_packet_core::{
    NetlinkMessage, NLM_F_ACK, NLM_F_DUMP, NLM_F_REQUEST,
};

use crate::{try_ethtool, EthtoolError, EthtoolHandle, EthtoolMessage};

pub struct FeatureGetRequest {
    handle: EthtoolHandle,
    iface_name: Option<String>,
}

impl FeatureGetRequest {
    pub(crate) fn new(handle: EthtoolHandle, iface_name: Option<&str>) -> Self {
        FeatureGetRequest {
            handle,
            iface_name: iface_name.map(|i| i.to_string()),
        }
    }

    pub fn execute(
        self,
    ) -> impl TryStream<Ok = EthtoolMessage, Error = EthtoolError> {
        let FeatureGetRequest {
            mut handle,
            iface_name,
        } = self;

        let nl_header_flags = match iface_name {
            None => NLM_F_DUMP | NLM_F_REQUEST | NLM_F_ACK,
            Some(_) => NLM_F_REQUEST,
        };

        let ethtool_msg = EthtoolMessage::new_feature_get(
            handle.family_id,
            iface_name.as_deref(),
        );

        let mut nl_msg = NetlinkMessage::from(ethtool_msg);

        nl_msg.header.flags = nl_header_flags;

        match handle.request(nl_msg) {
            Ok(response) => {
                Either::Left(response.map(move |msg| Ok(try_ethtool!(msg))))
            }
            Err(e) => Either::Right(
                futures::future::err::<EthtoolMessage, EthtoolError>(e)
                    .into_stream(),
            ),
        }
    }
}
