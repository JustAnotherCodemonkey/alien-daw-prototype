use crate::synth::{Clip, Mixer};
use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait},
    DefaultStreamConfigError, Device, DevicesError, Host, Stream, StreamConfig,
    SupportedStreamConfigsError,
};
use thiserror::Error;

pub struct Server {
    host: Host,
    output_device: Device,
    output_config: StreamConfig,
    output_stream: Stream,
    output_synth: Clip<Mixer>,
    input_device: Option<Device>,
    input_config: Option<StreamConfig>,
    input_stream: Option<Stream>,
}

impl Server {
    pub fn default_without_input() -> Result<Self, Error> {
        let host = default_host();
        let Some(output_device) = host.default_output_device() else {
            return Err(Error::NoDevices);
        };
        let input_stream_config = output_device.default_output_config()?;

        todo!();
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    DefaultStreamConfigError(#[from] DefaultStreamConfigError),
    #[error(transparent)]
    DevicesError(#[from] DevicesError),
    #[error("the host has no available devices")]
    NoDevices,
    #[error(transparent)]
    SupportedStreamConfigsError(#[from] SupportedStreamConfigsError),
}
