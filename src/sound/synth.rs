use std::fmt::Debug;
use thiserror::Error;

#[derive(Clone, Debug, Default)]
pub struct Clip<T: Synth> {
    pub inner: T,
    pub max_vol: VolumeControl,
}

impl<T: Synth> Synth for Clip<T> {
    fn sample(&mut self) -> f32 {
        self.inner
            .sample()
            .clamp(-self.max_vol.value(), self.max_vol.value())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Mixer {
    sub_synths: Vec<(SynthType, VolumeControl)>,
}

impl Synth for Mixer {
    fn sample(&mut self) -> f32 {
        self.sub_synths
            .iter_mut()
            .fold(0.0, |acc, synth| acc + synth.0.sample() * synth.1.value())
            / self.sub_synths.len() as f32
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct VolumeControl(f32);

impl VolumeControl {
    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for VolumeControl {
    fn default() -> Self {
        VolumeControl(1.0)
    }
}

impl TryFrom<f32> for VolumeControl {
    type Error = SynthError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_sign_negative() || !value.is_finite() {
            return Err(SynthError::NegativeVolume(value));
        }
        Ok(VolumeControl(value))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Silence;

impl Synth for Silence {
    fn sample(&mut self) -> f32 {
        0.0
    }
}

#[derive(Debug, Error)]
pub enum SynthError {
    #[error("VolumeControl value was negative, inf, or nan which are invalid states")]
    NegativeVolume(f32),
}

#[derive(Clone, Debug)]
pub enum SynthType {
    Clip(Clip<Box<SynthType>>),
    Mixer(Mixer),
    Silence,
}

impl Synth for SynthType {
    fn sample(&mut self) -> f32 {
        match self {
            SynthType::Clip(c) => c.sample(),
            SynthType::Mixer(m) => m.sample(),
            SynthType::Silence => Silence.sample(),
        }
    }
}

pub trait Synth: Clone {
    fn sample(&mut self) -> f32;
}

impl<T: Synth> Synth for Box<T> {
    fn sample(&mut self) -> f32 {
        (**self).sample()
    }
}
