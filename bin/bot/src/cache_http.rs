use std::{mem::MaybeUninit, sync::Arc};

pub struct CacheAndHttp();

static mut SINGLETON: MaybeUninit<Arc<serenity::CacheAndHttp>> = MaybeUninit::uninit();

impl CacheAndHttp {
    /// Gets a reference to the Bot cache and http
    ///
    /// # Panics
    ///
    /// Panics if the bot does not exists
    pub fn get() -> Arc<serenity::CacheAndHttp> {
        unsafe { SINGLETON.assume_init_ref().clone() }
    }

    /// Initializes the Bot cache and http
    pub fn init(bot: Arc<serenity::CacheAndHttp>) {
        unsafe {
            SINGLETON = MaybeUninit::new(bot);
        }
    }
}
