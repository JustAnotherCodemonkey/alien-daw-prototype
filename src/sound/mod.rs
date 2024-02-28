pub mod synth;

use crate::synth::{Clip, Mixer, Synth};
use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait},
    BuildStreamError, DefaultStreamConfigError, Device, DevicesError, FromSample, Host,
    OutputCallbackInfo, Sample, SampleFormat, SizedSample, Stream, StreamConfig, StreamError,
    SupportedStreamConfig, SupportedStreamConfigsError,
};
use either::Either;
use parking_lot::{FairMutex, FairMutexGuard};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

struct BackupFairMutex<T: Clone> {
    local: T,
    pub remote: Arc<FairMutex<T>>,
}

impl<T: Clone> BackupFairMutex<T> {
    fn get(&mut self) -> BackupFairMutexGuard<'_, T> {
        if let Some(g) = self.remote.try_lock() {
            self.local = g.clone();
            BackupFairMutexGuard::RemoteGuard(g)
        } else {
            BackupFairMutexGuard::LocalRef(&mut self.local)
        }
    }
}

impl<T: Clone> From<Arc<FairMutex<T>>> for BackupFairMutex<T> {
    fn from(value: Arc<FairMutex<T>>) -> Self {
        let local = value.lock().clone();
        BackupFairMutex {
            local,
            remote: value,
        }
    }
}

pub struct Server {
    host: Host,
    output_device: Device,
    output_config: StreamConfig,
    output_stream: Stream,
    output_synth: Arc<FairMutex<Clip<Mixer>>>,
    input_device: Option<Device>,
    input_config: Option<StreamConfig>,
    input_stream: Option<Stream>,
}

impl Server {
    pub fn default_without_input(
        error_handler_stream: UnboundedSender<StreamError>,
        callback_speed_warning_stream: Option<UnboundedSender<CallbackSpeedError>>,
    ) -> Result<Self, Error> {
        let host = default_host();
        let Some(output_device) = host.default_output_device() else {
            return Err(Error::NoDevices);
        };
        let supported_output_stream_config = output_device.default_output_config()?;
        let output_config = StreamConfig::from(supported_output_stream_config.clone());

        let prime_source = Arc::<FairMutex<Clip<Mixer>>>::default();

        let output_stream = build_output_stream(
            prime_source.clone(),
            callback_speed_warning_stream,
            supported_output_stream_config,
            error_handler_stream,
            &output_device,
        )?;

        Ok(Server {
            host,
            output_device,
            output_config,
            output_stream,
            output_synth: prime_source,
            input_device: None,
            input_config: None,
            input_stream: None,
        })
    }
}

enum BackupFairMutexGuard<'a, T> {
    LocalRef(&'a mut T),
    RemoteGuard(FairMutexGuard<'a, T>),
}

impl<'a, T> std::ops::Deref for BackupFairMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            BackupFairMutexGuard::LocalRef(r) => r,
            BackupFairMutexGuard::RemoteGuard(g) => &*g,
        }
    }
}

impl<'a, T> std::ops::DerefMut for BackupFairMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            BackupFairMutexGuard::LocalRef(r) => r,
            BackupFairMutexGuard::RemoteGuard(g) => &mut *g,
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    BuildStreamError(#[from] BuildStreamError),
    #[error(transparent)]
    CallbackSpeedError(#[from] CallbackSpeedError),
    #[error(transparent)]
    DefaultStreamConfigError(#[from] DefaultStreamConfigError),
    #[error(transparent)]
    DevicesError(#[from] DevicesError),
    #[error("the host has no available devices")]
    NoDevices,
    #[error(transparent)]
    SupportedStreamConfigsError(#[from] SupportedStreamConfigsError),
}

#[derive(Clone, Copy, Debug, Error)]
#[error("a stream data callback took too long to execute")]
pub struct CallbackSpeedError {
    actual_elapsed: Duration,
    max_elapsed: Duration,
    data_len: usize,
}

fn build_err_callback(
    err_channel: UnboundedSender<StreamError>,
) -> impl FnMut(StreamError) + Send + 'static {
    move |err| {
        // Err if reciever has been closed or dropped. In that case, we don't care.
        let _ = err_channel.send(err);
    }
}

fn build_output_callback<T: FromSample<f32> + Sample>(
    prime_source: Arc<FairMutex<Clip<Mixer>>>,
    callback_speed_warning_channel: Option<UnboundedSender<CallbackSpeedError>>,
) -> Either<
    impl FnMut(&mut [T], &OutputCallbackInfo) + Send + 'static,
    impl FnMut(&mut [T], &OutputCallbackInfo) + Send + 'static,
> {
    let mut backup_mutex = BackupFairMutex::from(prime_source);
    if let Some(channel) = callback_speed_warning_channel {
        Either::Left(move |data: &mut [T], callback_info: &OutputCallbackInfo| {
            let start_time = Instant::now();
            let mut lock = backup_mutex.get();
            for sample in data.iter_mut() {
                *sample = lock.sample().to_sample::<T>();
            }
            let actual_elapsed = start_time.elapsed();
            let max_elapsed = callback_info
                .timestamp()
                .playback
                .duration_since(&callback_info.timestamp().callback)
                .unwrap();
            if actual_elapsed > max_elapsed {
                // Err means that reciever was dropped or closed in which case we don't care.
                let _ = channel.send(CallbackSpeedError {
                    actual_elapsed,
                    max_elapsed,
                    data_len: data.len(),
                });
            }
        })
    } else {
        Either::Right(move |data: &mut [T], _: &_| {
            let mut lock = backup_mutex.get();
            for sample in data.iter_mut() {
                *sample = lock.sample().to_sample::<T>();
            }
        })
    }
}

fn build_output_stream(
    prime_source: Arc<FairMutex<Clip<Mixer>>>,
    callback_speed_warning_channel: Option<UnboundedSender<CallbackSpeedError>>,
    supported_stream_config: SupportedStreamConfig,
    err_channel: UnboundedSender<StreamError>,
    device: &Device,
) -> Result<Stream, Error> {
    let sample_format = supported_stream_config.sample_format();
    let stream_config = supported_stream_config.into();

    match sample_format {
        SampleFormat::F32 => build_output_stream_inner::<f32>(
            prime_source,
            callback_speed_warning_channel,
            &stream_config,
            err_channel,
            device,
        ),
        SampleFormat::F64 => build_output_stream_inner::<f64>(
            prime_source,
            callback_speed_warning_channel,
            &stream_config,
            err_channel,
            device,
        ),
        _ => unimplemented!("sample format of default output device not supported"),
    }
}

fn build_output_stream_inner<T: FromSample<f32> + Sample + SizedSample>(
    prime_source: Arc<FairMutex<Clip<Mixer>>>,
    callback_speed_warning_channel: Option<UnboundedSender<CallbackSpeedError>>,
    stream_config: &StreamConfig,
    err_channel: UnboundedSender<StreamError>,
    device: &Device,
) -> Result<Stream, Error> {
    let err_callback = build_err_callback(err_channel);
    match build_output_callback::<T>(prime_source, callback_speed_warning_channel) {
        Either::Left(callback) => {
            device.build_output_stream(stream_config, callback, err_callback, None)
        }
        Either::Right(callback) => {
            device.build_output_stream(stream_config, callback, err_callback, None)
        }
    }
    .map_err(|e| e.into())
}
