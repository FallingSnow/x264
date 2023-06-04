use crate::{Encoder, Encoding, Error, Result};
use core::ffi::c_char;
use core::mem::MaybeUninit;
use x264::*;

mod preset;
mod tune;

pub use self::preset::*;
pub use self::tune::*;

/// Builds a new encoder.
pub struct Setup {
    raw: x264_param_t,
}

impl Setup {
    /// Creates a new builder with the specified preset and tune.
    pub fn preset(preset: Preset, tune: Tune, fast_decode: bool, zero_latency: bool) -> Self {
        let mut raw = MaybeUninit::uninit();

        // Name validity verified at compile-time.
        assert_eq!(0, unsafe {
            x264_param_default_preset(
                raw.as_mut_ptr(),
                preset.to_cstr(),
                tune.to_cstr(fast_decode, zero_latency),
            )
        });

        Self {
            raw: unsafe { raw.assume_init() },
        }
    }

    /// Makes the first pass faster.
    pub fn fastfirstpass(mut self) -> Self {
        unsafe {
            x264_param_apply_fastfirstpass(&mut self.raw);
        }
        self
    }

    /// The video's framerate, represented as a rational number.
    ///
    /// The value is in frames per second.
    pub fn fps(mut self, num: u32, den: u32) -> Self {
        self.raw.i_fps_num = num;
        self.raw.i_fps_den = den;
        self
    }

    /// The encoder's timebase, used in rate control with timestamps.
    ///
    /// The value is in seconds per tick.
    pub fn timebase(mut self, num: u32, den: u32) -> Self {
        self.raw.i_timebase_num = num;
        self.raw.i_timebase_den = den;
        self
    }

    /// Enable/disable Annex B start codes. Defaults to `true`.
    ///
    /// Annex B start codes are not used by containers based on the ISO BMFF
    /// (Base Media File Format), such as MP4 and MOV.
    pub fn annexb(mut self, annexb: bool) -> Self {
        self.raw.b_annexb = annexb as i32;
        self
    }

    /// Approximately restricts the bitrate.
    ///
    /// The value is in metric kilobits per second. Setting this value also sets the rate control method to average bit rate and conflicts with setting crf.
    pub fn bitrate(mut self, average: i32) -> Self {
        self.raw.rc.i_rc_method = X264_RC_ABR as i32;
        self.raw.rc.i_bitrate = average;
        self
    }

    /// Target a constant rate factor. Contant rate factoring results in the best objective psnr/ssim per bit (efficiency).
    ///
    /// Values go from -12 to 51 with -12 resulting in the highest bitrate/quality. The default is `23.0`. This setting conflicts with setting bitrate.
    pub fn crf(mut self, target: f32, max: f32) -> Self {
        self.raw.rc.i_rc_method = X264_RC_CRF as i32;
        self.raw.rc.f_rf_constant = target;
        self.raw.rc.f_rf_constant_max = max;
        self
    }

    /// Restricts the maximum number of bframes.
    /// 
    /// Setting this to 0 disable bframes.
    pub fn bframes(mut self, max: i32) -> Self {
        self.raw.i_bframe = max;
        self
    }

    /// Sets the number of frames to be used as a buffer for threaded lookahead
    /// 
    /// 0 disables threaded lookahead, which allows lower latency at the cost of reduced efficiency
    pub fn lookahead(mut self, number: i32) -> Self {
        self.raw.i_sync_lookahead = number;
        self
    }

    /// When enabled, allows a group of pictures (GOP) to contain references to other groups of pictures
    pub fn open_gop(mut self, enabled: bool) -> Self {
        self.raw.b_open_gop = enabled as i32;
        self
    }

    /// The lowest profile, with guaranteed compatibility with all decoders.
    pub fn baseline(mut self) -> Self {
        unsafe {
            x264_param_apply_profile(&mut self.raw, b"baseline\0" as *const u8 as *const c_char);
        }
        self
    }

    /// A useless middleground between the baseline and high profiles.
    pub fn main(mut self) -> Self {
        unsafe {
            x264_param_apply_profile(&mut self.raw, b"main\0" as *const u8 as *const c_char);
        }
        self
    }

    /// The highest profile, which almost all encoders support.
    pub fn high(mut self) -> Self {
        unsafe {
            x264_param_apply_profile(&mut self.raw, b"high\0" as *const u8 as *const c_char);
        }
        self
    }

    /// Set the maximum number of frames between keyframes.
    pub fn max_keyframe_interval(mut self, interval: i32) -> Self {
        self.raw.i_keyint_max = interval;
        self
    }

    /// Set the minimum number of frames between keyframes.
    pub fn min_keyframe_interval(mut self, interval: i32) -> Self {
        self.raw.i_keyint_min = interval;
        self
    }

    /// Set the scenecut threshold. Set this to zero to guarantee a keyframe
    /// every `max_keyframe_interval`.
    pub fn scenecut_threshold(mut self, threshold: i32) -> Self {
        self.raw.i_scenecut_threshold = threshold;
        self
    }

    /// Build the encoder.
    pub fn build<C>(mut self, csp: C, width: i32, height: i32) -> Result<Encoder>
    where
        C: Into<Encoding>,
    {
        self.raw.i_csp = csp.into().into_raw();
        self.raw.i_width = width;
        self.raw.i_height = height;

        let raw = unsafe { x264_encoder_open(&mut self.raw) };

        if raw.is_null() {
            Err(Error)
        } else {
            Ok(unsafe { Encoder::from_raw(raw) })
        }
    }
}

impl Default for Setup {
    fn default() -> Self {
        let raw = unsafe {
            let mut raw = MaybeUninit::uninit();
            x264_param_default(raw.as_mut_ptr());
            raw.assume_init()
        };

        Self { raw }
    }
}
