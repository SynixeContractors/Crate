use std::mem::MaybeUninit;

use crate::ArcCacheAndHttp;

pub struct CacheAndHttp();

static mut SINGLETON: MaybeUninit<ArcCacheAndHttp> = MaybeUninit::uninit();

impl CacheAndHttp {
    /// Gets a reference to the Bot cache and http
    ///
    /// # Panics
    ///
    /// Panics if the bot does not exists
    pub fn get() -> ArcCacheAndHttp {
        unsafe { SINGLETON.assume_init_ref().clone() }
    }

    /// Initializes the Bot cache and http
    pub fn init(bot: ArcCacheAndHttp) {
        unsafe {
            SINGLETON = MaybeUninit::new(bot);
        }
    }
}
