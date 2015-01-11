/*!
 * Video recorder for the `kiss3d` graphics engine.
 */

#![crate_id = "kiss3d_recording#0.1"]
#![crate_type = "lib"]
#![deny(non_camel_case_types)]
#![deny(unnecessary_parens)]
#![deny(non_uppercase_statics)]
#![deny(unnecessary_qualification)]
#![warn(missing_doc)] // FIXME: should be denied.
#![deny(unused_result)]
#![deny(unnecessary_typecast)]
#![warn(visible_private_types)] // FIXME: should be denied.
#![feature(globs)]
#![feature(macro_rules)]
#![feature(managed_boxes)]
#![feature(unsafe_destructor)]
#![doc(html_root_url = "http://kiss3d.org/doc")]

extern crate std;
extern crate libc;
extern crate sync;
extern crate avcodec  = "avcodec55";
extern crate avutil   = "avutil52";
extern crate avformat = "avformat55";
extern crate swscale  = "swscale2";
extern crate kiss3d;

// inspired by the muxing sample: http://ffmpeg.org/doxygen/trunk/muxing_8c-source.html

use libc::c_void;
use swscale::Struct_SwsContext;
use avcodec::{AVCodec, AVCodecContext, AVPacket};
use avformat::{AVFormatContext, AVStream};
use avutil::{AVFrame, Struct_AVRational};
use std::ptr;
use std::mem;
use sync::one::{Once, ONCE_INIT};
use kiss3d::window::Window;

static mut avformat_init: Once = ONCE_INIT;

/// OpenGL rendering video recorder.
///
/// Use this to make a video of your crazy 3D scene.
pub struct Recorder {
    tmp_frame_buf:    Vec<u8>,
    frame_buf:        Vec<u8>,
    curr_frame_index: usize,
    initialized:      bool,
    bit_rate:         usize,
    width:            usize,
    height:           usize,
    time_base:        (usize, usize),
    gop_size:         usize,
    max_b_frames:     usize,
    pix_fmt:          i32,
    tmp_frame:        *mut AVFrame,
    frame:            *mut AVFrame,
    context:          *mut AVCodecContext,
    format_context:   *mut AVFormatContext,
    video_st:         *mut AVStream,
    scale_context:    *mut Struct_SwsContext,
    path:             Path
}

impl Recorder {
    /// Creates a new video recorder.
    ///
    /// # Arguments:
    /// * `path`   - path to the output file.
    /// * `width`  - width of the recorded video.
    /// * `height` - height of the recorded video.
    pub fn new(path: Path, width: usize, height: usize) -> Recorder {
        Recorder::new_with_params(path, width, height, None, None, None, None, None)
    }

    /// Creates a new video recorder with custom recording parameters.
    ///
    /// # Arguments:
    /// * `path`         - path to the output file.
    /// * `width`        - width of the recorded video.
    /// * `height`       - height of the recorded video.
    /// * `bit_rate`     - the average bit rate. Default value: 400000.
    /// * `time_base`    - this is the fundamental unit of time (in seconds) in terms of which
    ///                    frame timestamps are represented. Default value: (1, 60), i-e, 60fps.
    /// * `gop_size`     - the number of pictures in a group of pictures. Default value: 10.
    /// * `max_b_frames` - maximum number of B-frames between non-B-frames. Default value: 1.
    /// * `pix_fmt`      - pixel format. Default value: `avutil::PIX_FMT_YUV420P`.
    pub fn new_with_params(path:         Path,
                           width:        usize,
                           height:       usize,
                           bit_rate:     Option<usize>,
                           time_base:    Option<(usize, usize)>,
                           gop_size:     Option<usize>,
                           max_b_frames: Option<usize>,
                           pix_fmt:      Option<i32>)
                           -> Recorder {
        unsafe {
            avformat_init.doit(|| {
                avformat::av_register_all();
            });
        }

        let bit_rate     = bit_rate.unwrap_or(400000); // FIXME
        let time_base    = time_base.unwrap_or((1, 60));
        let gop_size     = gop_size.unwrap_or(10);
        let max_b_frames = max_b_frames.unwrap_or(1);
        let pix_fmt      = pix_fmt.unwrap_or(avutil::PIX_FMT_YUV420P);
        // width and height must be a multiple of two.
        let width        = if width  % 2 == 0 { width }  else { width + 1 };
        let height       = if height % 2 == 0 { height } else { height + 1 };

        Recorder {
            initialized:      false,
            curr_frame_index: 0,
            bit_rate:         bit_rate,
            width:            width,
            height:           height,
            time_base:        time_base,
            gop_size:         gop_size,
            max_b_frames:     max_b_frames,
            pix_fmt:          pix_fmt,
            frame:            ptr::mut_null(),
            tmp_frame:        ptr::mut_null(),
            context:          ptr::mut_null(),
            scale_context:    ptr::mut_null(),
            format_context:   ptr::mut_null(),
            video_st:         ptr::mut_null(),
            path:             path,
            frame_buf:        Vec::new(),
            tmp_frame_buf:    Vec::new()
        }
    }
                            
    /// Captures an image from the window and adds it to the current video.
    pub fn snap(&mut self, window: &Window) {
        self.init();

        let mut pkt: AVPacket = unsafe { mem::uninitialized() };

        unsafe {
            avcodec::av_init_packet(&mut pkt);
        }

        pkt.data = RawPtr::null();  // packet data will be allocated by the encoder
        pkt.size = 0;

        /*
         *
         * Fill the snapshot frame.
         *
         */
        window.snap(&mut self.tmp_frame_buf);


        let win_width  = window.width() as i32;
        let win_height = window.height() as i32;

        vflip(self.tmp_frame_buf.as_mut_slice(), win_width as usize * 3, win_height as usize);

        unsafe {
            (*self.frame).pts += avutil::av_rescale_q(1, (*self.context).time_base, (*self.video_st).time_base);
            self.curr_frame_index = self.curr_frame_index + 1;
        }

        unsafe {

            (*self.tmp_frame).width  = win_width;
            (*self.tmp_frame).height = win_height;

            let _ = avcodec::avpicture_fill(self.tmp_frame as *mut avcodec::AVPicture,
                                            self.tmp_frame_buf.get(0),
                                            avutil::PIX_FMT_RGB24,
                                            win_width,
                                            win_height);
        }

        /*
         * Convert the snapshot frame to the right format for the destination frame.
         */
        unsafe {
            self.scale_context = swscale::sws_getCachedContext(
                self.scale_context, win_width, win_height, avutil::PIX_FMT_RGB24,
                self.width as i32, self.height as i32, avutil::PIX_FMT_YUV420P,
                swscale::SWS_BICUBIC as i32, ptr::mut_null(), ptr::mut_null(), ptr::null()
                );

            let _ = swscale::sws_scale(self.scale_context,
                                       &(*self.tmp_frame).data[0], &(*self.tmp_frame).linesize[0],
                                       0, win_height,
                                       mem::transmute(&(*self.frame).data[0]), &(*self.frame).linesize[0]);
        }


        // Encode the image.

        let mut got_output = 0;
        let ret;

        unsafe {
            ret = avcodec::avcodec_encode_video2(self.context,
                                                 &mut pkt,
                                                 self.frame as *AVFrame,
                                                 &mut got_output);
        }

        if ret < 0 {
            panic!("Error encoding frame.");
        }

        if got_output != 0 {
            unsafe {
                let _ = avformat::av_interleaved_write_frame(self.format_context, &mut pkt);
                avcodec::av_free_packet(&mut pkt);
            }
        }
    }

    /// Initializes the recorder.
    ///
    /// This is automatically called when the first snapshot is made. Call this explicitly if you
    /// do not want the extra time overhead when the first snapshot is made.
    pub fn init(&mut self) {
        if self.initialized {
            return;
        }

        unsafe {
            // try to guess the container type from the path.
            let mut fmt = ptr::mut_null();

            self.path.with_c_str(|path| {
                let _ = avformat::avformat_alloc_output_context2(&mut fmt, ptr::mut_null(), ptr::null(), path);

                if self.format_context.is_null() {
                    // could not guess, default to MPEG
                    "mpeg".with_c_str(|mpeg| {
                        let _ = avformat::avformat_alloc_output_context2(&mut fmt, ptr::mut_null(), mpeg, path);
                    });
                }
            });

            self.format_context = fmt;

            if self.format_context.is_null() {
                panic!("Unable to create the output context.");
            }

            let fmt = (*self.format_context).oformat;

            if (*fmt).video_codec == avcodec::AV_CODEC_ID_NONE {
                panic!("The selected output container does not support video encoding.")
            }

            let mut codec: *mut AVCodec;

            let ret: i32 = 0;

            codec = avcodec::avcodec_find_encoder((*fmt).video_codec);

            if codec.is_null() {
                panic!("Codec not found.");
            }

            self.video_st = avformat::avformat_new_stream(self.format_context, codec as *AVCodec);

            if self.video_st.is_null() {
                panic!("Failed to allocate the video stream.");
            }

            (*self.video_st).id = ((*self.format_context).nb_streams - 1) as i32;

            self.context = (*self.video_st).codec;

            let _ = avcodec::avcodec_get_context_defaults3(self.context, codec as *AVCodec);

            if self.context.is_null() {
                panic!("Could not allocate video codec context.");
            }

            // sws scaling context
            self.scale_context = swscale::sws_getContext(
                self.width as i32, self.height as i32, avutil::PIX_FMT_RGB24,
                self.width as i32, self.height as i32, (*fmt).video_codec as i32,
                swscale::SWS_BICUBIC as i32, ptr::mut_null(), ptr::mut_null(), ptr::null());

            // Put sample parameters.
            (*self.context).bit_rate = self.bit_rate as i32;

            // Resolution must be a multiple of two.
            (*self.context).width    = self.width  as i32;
            (*self.context).height   = self.height as i32;

            // frames per second.
            let (tnum, tdenum)           = self.time_base;
            (*self.context).time_base    = Struct_AVRational { num: tnum as i32, den: tdenum as i32 };
            (*self.context).gop_size     = self.gop_size as i32;
            (*self.context).max_b_frames = self.max_b_frames as i32;
            (*self.context).pix_fmt      = self.pix_fmt;

            if (*self.context).codec_id == avcodec::AV_CODEC_ID_MPEG1VIDEO {
                // Needed to avoid using macroblocks in which some coeffs overflow.
                // This does not happen with normal video, it just happens here as
                // the motion of the chroma plane does not match the luma plane.
                (*self.context).mb_decision = 2;
            }

            /*
            if (*fmt).flags & avformat::AVFMT_GLOBALHEADER != 0 {
                (*self.context).flags = (*self.context).flags | CODEC_FLAG_GLOBAL_HEADER;
            }
            */

            // Open the codec.
            if avcodec::avcodec_open2(self.context, codec as *AVCodec, ptr::mut_null()) < 0 {
                panic!("Could not open the codec.");
            }

            /*
             * Init the destination video frame.
             */
            self.frame = avcodec::avcodec_alloc_frame();

            if self.frame.is_null() {
                panic!("Could not allocate the video frame.");
            }

            (*self.frame).format = (*self.context).pix_fmt;
            (*self.frame).width  = (*self.context).width;
            (*self.frame).height = (*self.context).height;
            (*self.frame).pts    = 0;

            // alloc the buffer
            let nframe_bytes = avcodec::avpicture_get_size(self.pix_fmt,
                                                           self.width as i32,
                                                           self.height as i32);
            self.frame_buf = Vec::from_elem(nframe_bytes as usize, 0u8);

            let _ = avcodec::avpicture_fill(self.frame as *mut avcodec::AVPicture,
                                            self.frame_buf.get(0),
                                            self.pix_fmt,
                                            self.width as i32,
                                            self.height as i32);

            /*
             * Init the temporary video frame.
             */
            self.tmp_frame = avcodec::avcodec_alloc_frame();

            if self.tmp_frame.is_null() {
                panic!("Could not allocate the video frame.");
            }

            (*self.frame).format = (*self.context).pix_fmt;
            // the rest (width, height, data, linesize) are set at the moment of the snapshot.

            // Open the output file.
            self.path.with_c_str(|path| {
                static AVIO_FLAG_WRITE: i32 = 2; // XXX: this should be defined by the bindings.
                if avformat::avio_open(&mut (*self.format_context).pb, path, AVIO_FLAG_WRITE) < 0 {
                    panic!("Failed to open the output file.");
                }
            });

            if avformat::avformat_write_header(self.format_context, ptr::mut_null()) < 0 {
                panic!("Failed to open the output file.");
            }

            if ret < 0 {
                panic!("Could not allocate raw picture buffer");
            }
        }

        self.initialized = true;
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        if self.initialized {
            // Get the delayed frames.
            let mut pkt:   AVPacket = unsafe { mem::uninitialized() };
            let mut got_output = 1;
            while got_output != 0 {
                let ret;

                unsafe {
                    avcodec::av_init_packet(&mut pkt);
                }

                pkt.data = RawPtr::null();  // packet data will be allocated by the encoder
                pkt.size = 0;

                unsafe {
                    ret = avcodec::avcodec_encode_video2(self.context, &mut pkt, ptr::null(), &mut got_output);
                }

                if ret < 0 {
                    panic!("Error encoding frame.");
                }

                if got_output != 0 {
                    unsafe {
                        let _ = avformat::av_interleaved_write_frame(self.format_context, &mut pkt);
                        avcodec::av_free_packet(&mut pkt);
                    }
                }
            }

            // Free things and stuffs.
            unsafe {
                let _ = avcodec::avcodec_close(self.context);
                avutil::av_free(self.context as *mut c_void);
                // avutil::av_freep((*self.frame).data[0] as *mut c_void);
                avcodec::avcodec_free_frame(&mut self.frame as *mut *mut AVFrame);
                avcodec::avcodec_free_frame(&mut self.tmp_frame as *mut *mut AVFrame);
            }
        }
    }
}

fn vflip(vec: &mut [u8], width: usize, height: usize) {
    for j in range(0u, height / 2) {
        for i in range(0u, width) {
            vec.swap((height - j - 1) * width + i, j * width + i);
        }
    }

}
