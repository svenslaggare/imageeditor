macro_rules! c_str {
    ($literal:expr) => {
        std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
    }
}

pub fn channels_type(num_channels: u32) -> u32 {
    match num_channels {
        4 => gl::RGBA,
        3 => gl::RGB,
        1 => gl::RED,
        _ => { panic!("Not supported number of channels..") }
    }
}