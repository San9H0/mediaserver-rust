use std::ffi::CString;
use std::ptr;
use anyhow::anyhow;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::context;
use std::ops::{Deref, DerefMut};
use std::mem::ManuallyDrop;
use crate::egress::sessions::hls::hls_data;
use crate::egress::servers::hls::HlsPayloader;


pub struct OutputWrap {
    output_ctx: ManuallyDrop<context::output::Output>,
}

impl HlsPayloader for OutputWrap {
    fn get_payload(&mut self) -> anyhow::Result<bytes::Bytes> {
        unsafe {
            let mut buffer: *mut u8 = ptr::null_mut();
            let pbuffer: *mut *mut u8 = &mut buffer;
            let size = ffmpeg_sys_next::avio_close_dyn_buf(
                (*self.output_ctx.as_mut_ptr()).pb,
                pbuffer);

            let data = std::slice::from_raw_parts(buffer, size as usize);

            let byte_buffer = bytes::Bytes::copy_from_slice(data);
            ffmpeg_sys_next::av_free(buffer as *mut libc::c_void);

            let mut avio_ctx = ptr::null_mut();
            if ffmpeg_sys_next::avio_open_dyn_buf(&mut avio_ctx) != 0 {
                return Err(anyhow!("avio_open_dyn_buf failed"));
            }

            let output_ctx = self.output_ctx.as_mut_ptr();
            (*output_ctx).pb = avio_ctx;

            Ok(byte_buffer)
        }
    }
}


impl OutputWrap {
    pub(crate) fn new() -> anyhow::Result<Self> {
        let filename = "output.mp4";
        let mut ps = ptr::null_mut();
        let path = CString::new(filename)?;
        let mut ctx = unsafe {
            let ret = ffmpeg_sys_next::avformat_alloc_output_context2(
                &mut ps,
                ptr::null_mut(),
                ptr::null(),
                path.as_ptr(),
            );
            if ret < 0 {
                return Err(anyhow!(ffmpeg::Error::from(ret).to_string()));
            }
            let mut avio_ctx = ptr::null_mut();
            let ret = ffmpeg_sys_next::avio_open_dyn_buf(&mut avio_ctx);
            if ret < 0 {
                return Err(anyhow!(ffmpeg::Error::from(ret).to_string()));
            }
            (*ps).pb = avio_ctx;
            context::Output::wrap(ps)
        };

        Ok(Self {
            output_ctx: ManuallyDrop::new(ctx),
        })
    }

    pub(crate) fn get_buffer(&mut self) -> anyhow::Result<bytes::Bytes> {
        unsafe {
            let mut buffer: *mut u8 = ptr::null_mut();
            let pbuffer: *mut *mut u8 = &mut buffer;
            let size = ffmpeg_sys_next::avio_close_dyn_buf((*self.output_ctx.as_mut_ptr()).pb, pbuffer);
            println!("size: {}", size);
            let data = std::slice::from_raw_parts(buffer, size as usize);

            let byte_buffer = bytes::Bytes::copy_from_slice(data);
            ffmpeg_sys_next::av_free(buffer as *mut libc::c_void);

            let mut avio_ctx = ptr::null_mut();
            if ffmpeg_sys_next::avio_open_dyn_buf(&mut avio_ctx) != 0 {
                return Err(anyhow!("avio_open_dyn_buf failed"));
            }

            let output_ctx = self.output_ctx.as_mut_ptr();
            (*output_ctx).pb = avio_ctx;

            Ok(byte_buffer)
        }
    }

    pub(crate) fn write_file(&mut self, filename: &str) -> anyhow::Result<()>{
        let mut buffer: *mut u8 = ptr::null_mut();
        let pbuffer: *mut *mut u8 = &mut buffer;
        unsafe {
            let size = ffmpeg_sys_next::avio_close_dyn_buf((*self.output_ctx.as_ptr()).pb, pbuffer);
            let data = std::slice::from_raw_parts(buffer, size as usize);
            let hls_data = hls_data::HlsData::from(data);
            let mut avio_ctx = ptr::null_mut();
            if ffmpeg_sys_next::avio_open_dyn_buf(&mut avio_ctx) < 0 {
                return Err(anyhow!("avio_open_dyn_buf failed"));
            }
            let format_ctx = self.output_ctx.as_mut_ptr();
            (*format_ctx).pb = avio_ctx;
            Ok(())
        }
    }
}

impl Deref for OutputWrap {
    type Target = context::output::Output;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.output_ctx }
    }
}

impl DerefMut for OutputWrap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut self.output_ctx }
    }
}

impl Drop for OutputWrap {
    fn drop(&mut self) {
        unsafe {
            let mut buffer: *mut u8 = ptr::null_mut();
            let pbuffer: *mut *mut u8 = &mut buffer;
            let size = ffmpeg_sys_next::avio_close_dyn_buf((*self.output_ctx.as_ptr()).pb, pbuffer);
            ffmpeg_sys_next::avformat_free_context(self.output_ctx.as_mut_ptr());
        }
    }
}