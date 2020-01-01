use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    fmt,
};
use num_traits::{FromPrimitive};
use oboe_sys as ffi;

use super::{
    Result,
    AudioApi,
    SharingMode,
    PerformanceMode,
    Usage,
    ContentType,
    InputPreset,
    SessionId,
    SampleRateConversionQuality,
    RawAudioStreamBase,

    Unspecified,
    IsDirection, Input, Output,
    IsFrameType,
    IsFormat,
    IsChannelCount, Mono, Stereo,

    AudioInputCallback,
    AudioOutputCallback,

    AudioCallbackWrapper,

    AudioStreamSync,
    AudioStreamAsync,

    wrap_status,
    audio_stream_base_fmt,
};

/**
 * Factory for an audio stream.
 */
#[repr(transparent)]
pub struct AudioStreamBuilder<D, C, T> {
    raw: ffi::oboe_AudioStreamBuilder,
    _phantom: PhantomData<(D, C, T)>,
}

impl<D, C, T> fmt::Debug for AudioStreamBuilder<D, C, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        audio_stream_base_fmt(self, f)
    }
}

impl<D, C, T> RawAudioStreamBase for AudioStreamBuilder<D, C, T> {
    fn _raw_base(&self) -> &ffi::oboe_AudioStreamBase {
        &self.raw._base
    }

    fn _raw_base_mut(&mut self) -> &mut ffi::oboe_AudioStreamBase {
        &mut self.raw._base
    }
}

impl AudioStreamBuilder<Output, Unspecified, Unspecified> {
    /**
     * Create new audio stream builder
     */
    pub fn new() -> Self {
        let mut raw = MaybeUninit::uninit();

        Self {
            raw: unsafe {
                ffi::oboe_AudioStreamBuilder_new(raw.as_mut_ptr());
                raw.assume_init()
            },
            _phantom: PhantomData,
        }
    }
}

impl<D, C, T> AudioStreamBuilder<D, C, T> {
    /**
     * Request a specific number of channels.
     *
     * Default is ChannelCount::Unspecified. If the value is unspecified then
     * the application should query for the actual value after the stream is opened.
     */
    pub fn set_channel_count<X: IsChannelCount>(mut self) -> AudioStreamBuilder<D, X, T> {
        self.raw._base.mChannelCount = X::CHANNEL_COUNT as i32;
        AudioStreamBuilder { raw: self.raw, _phantom: PhantomData }
    }

    pub fn set_mono(self) -> AudioStreamBuilder<D, Mono, T> {
        self.set_channel_count::<Mono>()
    }

    pub fn set_stereo(self) -> AudioStreamBuilder<D, Stereo, T> {
        self.set_channel_count::<Stereo>()
    }

    /**
     * Request the direction for a stream. The default is Direction::Output.
     *
     * @param direction Direction::Output or Direction::Input
     */
    pub fn set_direction<X: IsDirection>(mut self) -> AudioStreamBuilder<X, C, T> {
        self.raw._base.mDirection = X::DIRECTION as i32;
        AudioStreamBuilder { raw: self.raw, _phantom: PhantomData }
    }

    pub fn set_input(self) -> AudioStreamBuilder<Input, C, T> {
        self.set_direction::<Input>()
    }

    pub fn set_output(self) -> AudioStreamBuilder<Output, C, T> {
        self.set_direction::<Output>()
    }

    /**
     * Request a specific sample rate in Hz.
     *
     * Default is kUnspecified. If the value is unspecified then
     * the application should query for the actual value after the stream is opened.
     *
     * Technically, this should be called the "frame rate" or "frames per second",
     * because it refers to the number of complete frames transferred per second.
     * But it is traditionally called "sample rate". Se we use that term.
     *
     */
    pub fn set_sample_rate(&mut self, sample_rate: i32) -> &mut Self {
        self.raw._base.mSampleRate = sample_rate;
        self
    }

    /**
     * Request a specific number of frames for the data callback.
     *
     * Default is kUnspecified. If the value is unspecified then
     * the actual number may vary from callback to callback.
     *
     * If an application can handle a varying number of frames then we recommend
     * leaving this unspecified. This allow the underlying API to optimize
     * the callbacks. But if your application is, for example, doing FFTs or other block
     * oriented operations, then call this function to get the sizes you need.
     *
     * @param frames_per_callback
     * @return pointer to the builder so calls can be chained
     */
    pub fn set_frames_per_callback(&mut self, frames_per_callback: i32) -> &mut Self {
        self.raw._base.mFramesPerCallback = frames_per_callback;
        self
    }

    /**
     * Request a sample data format, for example Format::Float.
     *
     * Default is Format::Unspecified. If the value is unspecified then
     * the application should query for the actual value after the stream is opened.
     */
    pub fn set_format<X: IsFormat>(mut self) -> AudioStreamBuilder<D, C, X> {
        self.raw._base.mFormat = X::FORMAT as i32;
        AudioStreamBuilder { raw: self.raw, _phantom: PhantomData }
    }

    pub fn set_i16(self) -> AudioStreamBuilder<D, C, i16> {
        self.set_format::<i16>()
    }

    pub fn set_f32(self) -> AudioStreamBuilder<D, C, f32> {
        self.set_format::<f32>()
    }

    /**
     * Set the requested buffer capacity in frames.
     * BufferCapacityInFrames is the maximum possible BufferSizeInFrames.
     *
     * The final stream capacity may differ. For AAudio it should be at least this big.
     * For OpenSL ES, it could be smaller.
     *
     * Default is kUnspecified.
     *
     * @param bufferCapacityInFrames the desired buffer capacity in frames or kUnspecified
     * @return pointer to the builder so calls can be chained
     */
    pub fn set_buffer_capacity_in_frames(&mut self, buffer_capacity_in_frames: i32) -> &mut Self {
        self.raw._base.mBufferCapacityInFrames = buffer_capacity_in_frames;
        self
    }

    /**
     * Get the audio API which will be requested when opening the stream. No guarantees that this is
     * the API which will actually be used. Query the stream itself to find out the API which is
     * being used.
     *
     * If you do not specify the API, then AAudio will be used if isAAudioRecommended()
     * returns true. Otherwise OpenSL ES will be used.
     *
     * @return the requested audio API
     */
    pub fn get_audio_api(&self) -> AudioApi {
        FromPrimitive::from_i32(self.raw.mAudioApi).unwrap()
    }

    /**
     * If you leave this unspecified then Oboe will choose the best API
     * for the device and SDK version at runtime.
     *
     * This should almost always be left unspecified, except for debugging purposes.
     * Specifying AAudio will force Oboe to use AAudio on 8.0, which is extremely risky.
     * Specifying OpenSLES should mainly be used to test legacy performance/functionality.
     *
     * If the caller requests AAudio and it is supported then AAudio will be used.
     *
     * @param audioApi Must be AudioApi::Unspecified, AudioApi::OpenSLES or AudioApi::AAudio.
     * @return pointer to the builder so calls can be chained
     */
    pub fn set_audio_api(&mut self, audio_api: AudioApi) -> &mut Self {
        self.raw.mAudioApi = audio_api as i32;
        self
    }

    /**
     * Is the AAudio API supported on this device?
     *
     * AAudio was introduced in the Oreo 8.0 release.
     *
     * @return true if supported
     */
    pub fn is_aaudio_supported() -> bool {
        unsafe { ffi::oboe_AudioStreamBuilder_isAAudioSupported() }
    }

    /**
     * Is the AAudio API recommended this device?
     *
     * AAudio may be supported but not recommended because of version specific issues.
     * AAudio is not recommended for Android 8.0 or earlier versions.
     *
     * @return true if recommended
     */
    pub fn is_aaudio_recommended() -> bool {
        unsafe { ffi::oboe_AudioStreamBuilder_isAAudioRecommended() }
    }

    /**
     * Request a mode for sharing the device.
     * The requested sharing mode may not be available.
     * So the application should query for the actual mode after the stream is opened.
     *
     * @param sharingMode SharingMode::Shared or SharingMode::Exclusive
     * @return pointer to the builder so calls can be chained
     */
    pub fn set_sharing_mode(&mut self, sharing_mode: SharingMode) -> &mut Self {
        self.raw._base.mSharingMode = sharing_mode as i32;
        self
    }

    pub fn set_shared(&mut self) -> &mut Self {
        self.set_sharing_mode(SharingMode::Shared)
    }

    pub fn set_exclusive(&mut self) -> &mut Self {
        self.set_sharing_mode(SharingMode::Exclusive)
    }

    /**
     * Request a performance level for the stream.
     * This will determine the latency, the power consumption, and the level of
     * protection from glitches.
     *
     * @param performanceMode for example, PerformanceMode::LowLatency
     * @return pointer to the builder so calls can be chained
     */
    pub fn set_performance_mode(&mut self, performance_mode: PerformanceMode) -> &mut Self {
        self.raw._base.mPerformanceMode = performance_mode as i32;
        self
    }

    /**
     * Set the intended use case for the stream.
     *
     * The system will use this information to optimize the behavior of the stream.
     * This could, for example, affect how volume and focus is handled for the stream.
     *
     * The default, if you do not call this function, is Usage::Media.
     *
     * Added in API level 28.
     *
     * @param usage the desired usage, eg. Usage::Game
     */
    pub fn set_usage(&mut self, usage: Usage) -> &mut Self {
        self.raw._base.mUsage = usage as i32;
        self
    }

    /**
     * Set the type of audio data that the stream will carry.
     *
     * The system will use this information to optimize the behavior of the stream.
     * This could, for example, affect whether a stream is paused when a notification occurs.
     *
     * The default, if you do not call this function, is ContentType::Music.
     *
     * Added in API level 28.
     *
     * @param contentType the type of audio data, eg. ContentType::Speech
     */
    pub fn set_content_type(&mut self, content_type: ContentType) -> &mut Self {
        self.raw._base.mContentType = content_type as i32;
        self
    }

    /**
     * Set the input (capture) preset for the stream.
     *
     * The system will use this information to optimize the behavior of the stream.
     * This could, for example, affect which microphones are used and how the
     * recorded data is processed.
     *
     * The default, if you do not call this function, is InputPreset::VoiceRecognition.
     * That is because VoiceRecognition is the preset with the lowest latency
     * on many platforms.
     *
     * Added in API level 28.
     *
     * @param inputPreset the desired configuration for recording
     */
    pub fn set_input_preset(&mut self, input_preset: InputPreset) -> &mut Self {
        self.raw._base.mInputPreset = input_preset as i32;
        self
    }

    /**
     * Set the requested session ID.
     *
     * The session ID can be used to associate a stream with effects processors.
     * The effects are controlled using the Android AudioEffect Java API.
     *
     * The default, if you do not call this function, is SessionId::None.
     *
     * If set to SessionId::Allocate then a session ID will be allocated
     * when the stream is opened.
     *
     * The allocated session ID can be obtained by calling AudioStream::getSessionId()
     * and then used with this function when opening another stream.
     * This allows effects to be shared between streams.
     *
     * Session IDs from Oboe can be used the Android Java APIs and vice versa.
     * So a session ID from an Oboe stream can be passed to Java
     * and effects applied using the Java AudioEffect API.
     *
     * Allocated session IDs will always be positive and nonzero.
     *
     * Added in API level 28.
     *
     * @param sessionId an allocated sessionID or SessionId::Allocate
     */
    pub fn set_session_id(&mut self, session_id: SessionId) -> &mut Self {
        self.raw._base.mSessionId = session_id as i32;
        self
    }

    /**
     * Request a stream to a specific audio input/output device given an audio device ID.
     *
     * In most cases, the primary device will be the appropriate device to use, and the
     * deviceId can be left kUnspecified.
     *
     * On Android, for example, the ID could be obtained from the Java AudioManager.
     * AudioManager.getDevices() returns an array of AudioDeviceInfo[], which contains
     * a getId() method (as well as other type information), that should be passed
     * to this method.
     *
     *
     * Note that when using OpenSL ES, this will be ignored and the created
     * stream will have deviceId kUnspecified.
     *
     * @param deviceId device identifier or kUnspecified
     * @return pointer to the builder so calls can be chained
     */
    pub fn set_device_id(&mut self, device_id: i32) -> &mut Self {
        self.raw._base.mDeviceId = device_id;
        self
    }

    /**
     * If true then Oboe might convert channel counts to achieve optimal results.
     * On some versions of Android for example, stereo streams could not use a FAST track.
     * So a mono stream might be used instead and duplicated to two channels.
     * On some devices, mono streams might be broken, so a stereo stream might be opened
     * and converted to mono.
     *
     * Default is true.
     */
    pub fn set_channel_conversion_allowed(&mut self, allowed: bool) -> &mut Self {
        self.raw._base.mChannelConversionAllowed = allowed;
        self
    }

    /**
     * If true then  Oboe might convert data formats to achieve optimal results.
     * On some versions of Android, for example, a float stream could not get a
     * low latency data path. So an I16 stream might be opened and converted to float.
     *
     * Default is true.
     */
    pub fn set_format_conversion_allowed(&mut self, allowed: bool) -> &mut Self {
        self.raw._base.mFormatConversionAllowed = allowed;
        self
    }

    /**
     * Specify the quality of the sample rate converter in Oboe.
     *
     * If set to None then Oboe will not do sample rate conversion. But the underlying APIs might
     * still do sample rate conversion if you specify a sample rate.
     * That can prevent you from getting a low latency stream.
     *
     * If you do the conversion in Oboe then you might still get a low latency stream.
     *
     * Default is SampleRateConversionQuality::None
     */
    pub fn set_sample_rate_conversion_quality(&mut self, quality: SampleRateConversionQuality) -> &mut Self {
        self.raw._base.mSampleRateConversionQuality = quality as i32;
        self
    }

    /**
     * @return true if AAudio will be used based on the current settings.
     */
    pub fn will_use_aaudio(&self) -> bool {
        (self.raw.mAudioApi == (AudioApi::AAudio as i32) && Self::is_aaudio_supported()) ||
            (self.raw.mAudioApi == (AudioApi::Unspecified as i32) && Self::is_aaudio_recommended())
    }
}

impl<D: IsDirection, C: IsChannelCount, T: IsFormat> AudioStreamBuilder<D, C, T> {
    /**
     * Create and open a stream object based on the current settings.
     *
     * The caller owns the pointer to the AudioStream object.
     *
     * @param stream pointer to a variable to receive the stream address
     * @return OBOE_OK if successful or a negative error code
     */
    pub fn open_stream(mut self) -> Result<AudioStreamSync<D, (T, C)>> {
        let mut stream = MaybeUninit::<*mut ffi::oboe_AudioStream>::uninit();

        wrap_status(unsafe {
            ffi::oboe_AudioStreamBuilder_openStream(
                &mut self.raw,
                stream.as_mut_ptr(),
            )
        }).map(|_| AudioStreamSync::wrap_raw(
            unsafe { stream.assume_init() },
        ))
    }
}

impl<C: IsChannelCount, T: IsFormat> AudioStreamBuilder<Input, C, T> {
    /**
     * Specifies an object to handle data or error related callbacks from the underlying API.
     *
     * <strong>Important: See AudioStreamCallback for restrictions on what may be called
     * from the callback methods.</strong>
     *
     * When an error callback occurs, the associated stream will be stopped and closed in a separate thread.
     *
     * A note on why the streamCallback parameter is a raw pointer rather than a smart pointer:
     *
     * The caller should retain ownership of the object streamCallback points to. At first glance weak_ptr may seem like
     * a good candidate for streamCallback as this implies temporary ownership. However, a weak_ptr can only be created
     * from a shared_ptr. A shared_ptr incurs some performance overhead. The callback object is likely to be accessed
     * every few milliseconds when the stream requires new data so this overhead is something we want to avoid.
     *
     * This leaves a raw pointer as the logical type choice. The only caveat being that the caller must not destroy
     * the callback before the stream has been closed.
     *
     * @param streamCallback
     * @return pointer to the builder so calls can be chained
     */
    pub fn set_callback<F>(mut self, stream_callback: F) -> AudioStreamBuilderAsync<Input, F>
    where
        F: AudioInputCallback<FrameType = (T, C)>,
        (T, C): IsFrameType,
    {
        let callback = AudioCallbackWrapper::<Input, F>::wrap(stream_callback);
        self.raw._base.mStreamCallback = &callback.raw_callback() as *const _ as *mut _;
        AudioStreamBuilderAsync { raw: self.raw, callback, _phantom: PhantomData }
    }
}

impl<C: IsChannelCount, T: IsFormat> AudioStreamBuilder<Output, C, T> {
    /**
     * Specifies an object to handle data or error related callbacks from the underlying API.
     *
     * <strong>Important: See AudioStreamCallback for restrictions on what may be called
     * from the callback methods.</strong>
     *
     * When an error callback occurs, the associated stream will be stopped and closed in a separate thread.
     *
     * A note on why the streamCallback parameter is a raw pointer rather than a smart pointer:
     *
     * The caller should retain ownership of the object streamCallback points to. At first glance weak_ptr may seem like
     * a good candidate for streamCallback as this implies temporary ownership. However, a weak_ptr can only be created
     * from a shared_ptr. A shared_ptr incurs some performance overhead. The callback object is likely to be accessed
     * every few milliseconds when the stream requires new data so this overhead is something we want to avoid.
     *
     * This leaves a raw pointer as the logical type choice. The only caveat being that the caller must not destroy
     * the callback before the stream has been closed.
     *
     * @param streamCallback
     * @return pointer to the builder so calls can be chained
     */
    pub fn set_callback<F>(mut self, stream_callback: F) -> AudioStreamBuilderAsync<Output, F>
    where
        F: AudioOutputCallback<FrameType = (T, C)>,
        (T, C): IsFrameType,
    {
        let callback = AudioCallbackWrapper::<Output, F>::wrap(stream_callback);
        self.raw._base.mStreamCallback = &callback.raw_callback() as *const _ as *mut _;
        AudioStreamBuilderAsync { raw: self.raw, callback, _phantom: PhantomData }
    }
}

/**
 * Factory for an audio stream.
 */
pub struct AudioStreamBuilderAsync<D, F> {
    raw: ffi::oboe_AudioStreamBuilder,
    callback: AudioCallbackWrapper<D, F>,
    _phantom: PhantomData<(D, F)>,
}

impl<D, F> fmt::Debug for AudioStreamBuilderAsync<D, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        audio_stream_base_fmt(self, f)
    }
}

impl<D, F> RawAudioStreamBase for AudioStreamBuilderAsync<D, F> {
    fn _raw_base(&self) -> &ffi::oboe_AudioStreamBase {
        &self.raw._base
    }

    fn _raw_base_mut(&mut self) -> &mut ffi::oboe_AudioStreamBase {
        &mut self.raw._base
    }
}

impl<F: AudioInputCallback + Send> AudioStreamBuilderAsync<Input, F> {
    /**
     * Create and open a stream object based on the current settings.
     *
     * The caller owns the pointer to the AudioStream object.
     *
     * @param stream pointer to a variable to receive the stream address
     * @return OBOE_OK if successful or a negative error code
     */
    pub fn open_stream(mut self) -> Result<AudioStreamAsync<Input, F>> {
        let mut stream = MaybeUninit::<*mut ffi::oboe_AudioStream>::uninit();

        wrap_status(unsafe {
            ffi::oboe_AudioStreamBuilder_openStream(
                &mut self.raw,
                stream.as_mut_ptr(),
            )
        }).map(|_| AudioStreamAsync::wrap_raw(
            unsafe { stream.assume_init() },
            self.callback,
        ))
    }
}

impl<F: AudioOutputCallback + Send> AudioStreamBuilderAsync<Output, F> {
    /**
     * Create and open a stream object based on the current settings.
     *
     * The caller owns the pointer to the AudioStream object.
     *
     * @param stream pointer to a variable to receive the stream address
     * @return OBOE_OK if successful or a negative error code
     */
    pub fn open_stream(mut self) -> Result<AudioStreamAsync<Output, F>> {
        let mut stream = MaybeUninit::<*mut ffi::oboe_AudioStream>::uninit();

        wrap_status(unsafe {
            ffi::oboe_AudioStreamBuilder_openStream(
                &mut self.raw,
                stream.as_mut_ptr(),
            )
        }).map(|_| AudioStreamAsync::wrap_raw(
            unsafe { stream.assume_init() },
            self.callback,
        ))
    }
}
