use std::mem::MaybeUninit;

use tera::Tera;

pub struct Template;

static mut SINGLETON: MaybeUninit<Tera> = MaybeUninit::uninit();
static mut IS_INIT: bool = false;

impl Template {
    /// Gets a reference to the Template singleton
    ///
    /// # Panics
    ///
    /// Panics if the bot does not exists
    pub fn get<'a>() -> &'a Tera {
        unsafe {
            if !IS_INIT {
                Self::init();
            }
            SINGLETON.assume_init_ref()
        }
    }

    /// Initializes the Template singleton
    pub fn init() {
        unsafe {
            SINGLETON = MaybeUninit::new(
                Tera::new("templates/**/*.html").expect("Failed to load templates"),
            );
            IS_INIT = true;
        }
    }
}
