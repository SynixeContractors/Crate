use std::{mem::MaybeUninit, sync::Arc};

use serenity::gateway::ShardMessenger;

pub struct Bot();

static mut SINGLETON: MaybeUninit<Arc<ShardMessenger>> = MaybeUninit::uninit();

impl Bot {
    /// Gets a reference to the Bot cache and http
    ///
    /// # Panics
    ///
    /// Panics if the bot does not exists
    pub fn get() -> Arc<ShardMessenger> {
        unsafe { SINGLETON.assume_init_ref().clone() }
    }

    /// Initializes the Bot cache and http
    pub fn init(bot: Arc<ShardMessenger>) {
        unsafe {
            SINGLETON = MaybeUninit::new(bot);
        }
    }
}
