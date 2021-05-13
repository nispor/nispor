// Copyright 2021 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod coalesce;
mod connection;
mod error;
mod feature;
mod handle;
mod header;
mod macros;
mod message;
mod pause;
mod ring;

pub use coalesce::{CoalesceAttr, CoalesceGetRequest, CoalesceHandle};
pub use connection::new_connection;
pub use error::EthtoolError;
pub use feature::{FeatureAttr, FeatureBit, FeatureGetRequest, FeatureHandle};
pub use handle::EthtoolHandle;
pub use header::EthtoolHeader;
pub use message::{EthoolAttr, EthtoolMessage};
pub use pause::{PauseAttr, PauseGetRequest, PauseHandle};
pub use ring::{RingAttr, RingGetRequest, RingHandle};
