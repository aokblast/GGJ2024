use dlopen2::wrapper::{Container, WrapperApi};
use std::thread;
use std::time::Duration;

#[derive(WrapperApi)]
pub struct RingConApi {
    ringcon_init: unsafe extern "C" fn(),
    poll_ringcon: unsafe extern "C" fn(pull_val: *mut PullVal),
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct PullVal {
    pub running: bool,
    pub squatting: bool,
    pub push_val: i32,
}

pub(crate) fn test_ringcon() {
    let ringcon_api: Container<RingConApi> =
        unsafe { Container::load("./ringcon_driver.dll") }.unwrap();

    unsafe {
        ringcon_api.ringcon_init();
    }

    let mut status = PullVal::default();

    loop {
        unsafe {
            ringcon_api.poll_ringcon(unsafe { &mut status } as *mut PullVal);
        }
        let val = status.push_val;
        println!("{}", val);
        thread::sleep(Duration::from_millis(24));
    }
}
