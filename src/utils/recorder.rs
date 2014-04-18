use libc::c_void;
use std::cast;
use swscale;
use swscale::Struct_SwsContext;
use avcodec;
use avcodec::{AVCodec, AVCodecContext, AVPacket, Enum_AVCodecID};
use avutil;
use avutil::{AVFrame, Struct_AVRational};
use std::slice::raw;
use std::io::File;
use std::ptr;
use std::mem;
use sync::one::{Once, ONCE_INIT};
use window::Window;

static mut avcodec_init: Once = ONCE_INIT;

/// OpenGL rendering video recorder.
///
/// Use this to make a video of your crazy 3D scene.
pub struct Recorder {
    tmp_frame_buf:    Vec<u8>,
    frame_buf:        Vec<u8>,
    curr_frame_index: uint,
    initialized:      bool,
    codec_id:         Enum_AVCodecID,
    bit_rate:         uint,
    width:            uint,
    height:           uint,
    time_base:        (uint, uint),
    gop_size:         uint,
    max_b_frames:     uint,
    pix_fmt:          i32,
    tmp_frame:        *mut AVFrame,
    frame:            *mut AVFrame,
    context:          *mut AVCodecContext,
    scale_context:    *mut Struct_SwsContext,
    file:             File
}

impl Recorder {
    /// Creates a new video recorder.
    ///
    /// # Arguments:
    /// * `path`   - path to the output file.
    /// * `width`  - width of the recorded video.
    /// * `height` - height of the recorded video.
    pub fn new(path: &Path, width: uint, height: uint) -> Recorder {
        // FIXME: Use a default codec that we are sure the user has.
        Recorder::new_with_params(path, width, height, None, None, None, None, None, None)
    }

    /// Creates a new video recorder with custom recording parameters.
    ///
    /// # Arguments:
    /// * `path`         - path to the output file.
    /// * `width`        - width of the recorded video.
    /// * `height`       - height of the recorded video.
    /// * `codec`        - the codec used to encode the video. Default value: `avcodec::AV_CODEC_ID_H264`.
    /// * `bit_rate`     - the average bit rate. Default value: 400000.
    /// * `time_base`    - this is the fundamental unit of time (in seconds) in terms of which
    ///                    frame timestamps are represented. Default value: (1, 25), i-e, 25fps.
    /// * `gop_size`     - the number of pictures in a group of pictures. Default value: 10.
    /// * `max_b_frames` - maximum number of B-frames between non-B-frames. Default value: 1.
    /// * `pix_fmt`      - pixel format. Default value: `avcodec::AV_CODEC_ID_H264`.
    pub fn new_with_params(path:         &Path,
                           width:        uint,
                           height:       uint,
                           codec:        Option<Enum_AVCodecID>,
                           bit_rate:     Option<uint>,
                           time_base:    Option<(uint, uint)>,
                           gop_size:     Option<uint>,
                           max_b_frames: Option<uint>,
                           pix_fmt:      Option<i32>)
                           -> Recorder {
        unsafe {
            avcodec_init.doit(|| {
                avcodec::avcodec_register_all();
            });
        }

        let codec        = codec.unwrap_or(avcodec::AV_CODEC_ID_H264);
        let bit_rate     = bit_rate.unwrap_or(400000); // FIXME
        let time_base    = time_base.unwrap_or((1, 25));
        let gop_size     = gop_size.unwrap_or(10);
        let max_b_frames = max_b_frames.unwrap_or(1);
        let pix_fmt      = pix_fmt.unwrap_or(avutil::PIX_FMT_YUV420P);
        // width and height must be a multiple of two.
        let width        = if width  % 2 == 0 { width }  else { width + 1 };
        let height       = if height % 2 == 0 { height } else { height + 1 };

        let file =
            match File::create(path) {
                Ok(file) => file,
                Err(e)   => fail!(e)
            };

        let nframe_bytes;
        let nframe_tmp_bytes;

        unsafe {
            nframe_bytes     = avcodec::avpicture_get_size(avutil::PIX_FMT_YUV420P, width as i32, height as i32);
            nframe_tmp_bytes = avcodec::avpicture_get_size(avutil::PIX_FMT_RGB24, width as i32, height as i32);
        }

        Recorder {
            initialized:      false,
            curr_frame_index: 0,
            codec_id:         codec,
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
            file:             file,
            // XXX: we do the following hacky allocation since the bindings do not give
            // access to avcodec::av_image_alloc…
            // FIXME: does that depend on the pix_fmt ?
            frame_buf:        Vec::from_elem(nframe_bytes as uint, 0u8),
            tmp_frame_buf:    Vec::from_elem(nframe_tmp_bytes as uint, 0u8)
        }
    }
                            
    /// Captures an image from the window and adds it to the current video.
    pub fn snap(&mut self, window: &Window) {
        self.init();

        let mut pkt: AVPacket = unsafe { mem::uninit() };

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

        unsafe {
            (*self.frame).pts = self.curr_frame_index as i64;
            (*self.frame).pkt_duration = self.curr_frame_index as i64;
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
                                       cast::transmute(&(*self.frame).data[0]), &(*self.frame).linesize[0]);
        }


        /* */

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
            fail!("Error encoding frame.");
        }

        if got_output != 0 {
            unsafe {
                let _ = raw::buf_as_slice(pkt.data as *u8, pkt.size as uint, |data| self.file.write(data));
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

        let mut codec: *mut AVCodec;

        let ret: i32 = 0;

        unsafe {
            codec = avcodec::avcodec_find_encoder(self.codec_id);
        }

        if codec.is_null() {
            fail!("Codec not found.");
        }

        unsafe {
            self.context = avcodec::avcodec_alloc_context3(codec as *AVCodec);
        }

        if self.context.is_null() {
            fail!("Could not allocate video codec context.");
        }

        // sws scaling context
        unsafe {
            self.scale_context = swscale::sws_getContext(
                self.width as i32, self.height as i32, avutil::PIX_FMT_RGB24,
                self.width as i32, self.height as i32, avutil::PIX_FMT_YUV420P,
                swscale::SWS_BICUBIC as i32, ptr::mut_null(), ptr::mut_null(), ptr::null());
        }

        // Put sample parameters.
        unsafe {
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
        }

        if self.codec_id == avcodec::AV_CODEC_ID_H264 {
            "preset".to_c_str().with_ref(|preset| {
                "slow".to_c_str().with_ref(|slow| {
                    unsafe {
                        let _ = avutil::av_opt_set((*self.context).priv_data, preset, slow, 0);
                    }
                })
            });
        }

        // Open it.
        unsafe {
            if avcodec::avcodec_open2(self.context, codec as *AVCodec, ptr::mut_null()) < 0 {
                fail!("Could not open the codec.");
            }
        }

        /*
         * Init the destination video frame.
         */
        unsafe {
            self.frame = avcodec::avcodec_alloc_frame();
        }

        if self.frame.is_null() {
            fail!("Could not allocate the video frame.");
        }

        unsafe {
            (*self.frame).format = (*self.context).pix_fmt;
            (*self.frame).width  = (*self.context).width;
            (*self.frame).height = (*self.context).height;

            let _ = avcodec::avpicture_fill(self.frame as *mut avcodec::AVPicture,
                                            self.frame_buf.get(0),
                                            avutil::PIX_FMT_YUV420P,
                                            self.width as i32,
                                            self.height as i32);
        }

        /*
         * Init the temporary video frame.
         */
        unsafe {
            self.tmp_frame = avcodec::avcodec_alloc_frame();
        }

        if self.tmp_frame.is_null() {
            fail!("Could not allocate the video frame.");
        }

        unsafe {
            (*self.frame).format = (*self.context).pix_fmt;
            // the rest (width, height, data, linesize) are set at the moment of the snapshot.
        }

        if ret < 0 {
            fail!("Could not allocate raw picture buffer");
        }

        self.initialized = true;
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        if self.initialized {
            // Get the delayed frames.
            let mut pkt:   AVPacket = unsafe { mem::uninit() };
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
                    fail!("Error encoding frame.");
                }

                if got_output != 0 {
                    unsafe {
                        let _ = raw::buf_as_slice(pkt.data as *u8, pkt.size as uint, |data| self.file.write(data));
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
