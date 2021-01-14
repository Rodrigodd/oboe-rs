use num_derive::FromPrimitive;

use crate::AudioFormat;

pub(crate) struct Context;

impl Context {
    pub const AUDIO_SERVICE: &'static str = "audio";
}

pub(crate) struct PackageManager;

impl PackageManager {
    pub const FEATURE_AUDIO_LOW_LATENCY: &'static str = "android.hardware.audio.low_latency";
    pub const FEATURE_AUDIO_OUTPUT: &'static str = "android.hardware.audio.output";
    pub const FEATURE_AUDIO_PRO: &'static str = "android.hardware.audio.pro";
    pub const FEATURE_MICROPHONE: &'static str = "android.hardware.microphone";
    pub const FEATURE_MIDI: &'static str = "android.software.midi";
}

pub(crate) struct AudioManager;

impl AudioManager {
    pub const PROPERTY_OUTPUT_SAMPLE_RATE: &'static str =
        "android.media.property.OUTPUT_SAMPLE_RATE";
    pub const PROPERTY_OUTPUT_FRAMES_PER_BUFFER: &'static str =
        "android.media.property.OUTPUT_FRAMES_PER_BUFFER";

    pub const GET_DEVICES_INPUTS: i32 = 1 << 0;
    pub const GET_DEVICES_OUTPUTS: i32 = 1 << 1;
    pub const GET_DEVICES_ALL: i32 = Self::GET_DEVICES_INPUTS | Self::GET_DEVICES_OUTPUTS;
}

/**
 * The Android audio device info
 */
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "java-interface")))]
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    /**
     * Device identifier
     */
    pub id: i32,

    /**
     * The type of device
     */
    pub device_type: AudioDeviceType,

    /**
     * The device can be used for playback and/or capture
     */
    pub direction: AudioDeviceDirection,

    /**
     * Device address
     */
    pub address: String,

    /**
     * Device product name
     */
    pub product_name: String,

    /**
     * Available channel configurations
     */
    pub channel_counts: Vec<i32>,

    /**
     * Supported sample rates
     */
    pub sample_rates: Vec<i32>,

    /**
     * Supported audio formats
     */
    pub formats: Vec<AudioFormat>,
}

/**
 * The type of audio device
 */
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "java-interface")))]
#[derive(Debug, Clone, Copy, FromPrimitive)]
#[non_exhaustive]
#[repr(i32)]
pub enum AudioDeviceType {
    Unknown = 0,
    AuxLine = 19,
    BluetoothA2DP = 8,
    BluetoothSCO = 7,
    BuiltinEarpiece = 1,
    BuiltinMic = 15,
    BuiltinSpeaker = 2,
    BuiltinSpeakerSafe = 24,
    Bus = 21,
    Dock = 13,
    Fm = 14,
    FmTuner = 16,
    Hdmi = 9,
    HdmiArc = 10,
    HearingAid = 23,
    Ip = 20,
    LineAnalog = 5,
    LineDigital = 6,
    Telephony = 18,
    TvTuner = 17,
    UsbAccessory = 12,
    UsbDevice = 11,
    UsbHeadset = 22,
    WiredHeadphones = 4,
    WiredHeadset = 3,
}

/**
 * The direction of audio device
 */
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "java-interface")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum AudioDeviceDirection {
    Input = AudioManager::GET_DEVICES_INPUTS,
    Output = AudioManager::GET_DEVICES_OUTPUTS,
    InputOutput = AudioManager::GET_DEVICES_ALL,
}

impl AudioDeviceDirection {
    pub fn new(is_input: bool, is_output: bool) -> Option<Self> {
        use self::AudioDeviceDirection::*;
        match (is_input, is_output) {
            (true, true) => Some(InputOutput),
            (false, true) => Some(Output),
            (true, false) => Some(Input),
            _ => None,
        }
    }

    pub fn is_input(&self) -> bool {
        *self != AudioDeviceDirection::Output
    }

    pub fn is_output(&self) -> bool {
        *self != AudioDeviceDirection::Input
    }
}

impl AudioFormat {
    pub(crate) const ENCODING_PCM_16BIT: i32 = 2;
    //pub(crate) const ENCODING_PCM_8BIT: i32 = 3;
    pub(crate) const ENCODING_PCM_FLOAT: i32 = 4;

    pub(crate) fn from_encoding(encoding: i32) -> Option<AudioFormat> {
        match encoding {
            AudioFormat::ENCODING_PCM_16BIT => Some(AudioFormat::I16),
            AudioFormat::ENCODING_PCM_FLOAT => Some(AudioFormat::F32),
            _ => None,
        }
    }
}
