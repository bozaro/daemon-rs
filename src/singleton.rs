#![macro_use]
/**
 * Thanks for http://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
 */
use std::sync::{Arc, LockResult, Mutex, MutexGuard};

//#[derive(Copy)]
pub struct SingletonHolder<T> {
    // Since we will be used in many threads, we need to protect
    // concurrent access
    inner: Arc<Mutex<T>>,
}

impl<T> SingletonHolder<T> {
    pub fn new(mutex: Arc<Mutex<T>>) -> SingletonHolder<T> {
        SingletonHolder { inner: mutex }
    }

    pub fn lock(&self) -> LockResult<MutexGuard<T>> {
        self.inner.lock()
    }
}

impl<T> Clone for SingletonHolder<T> {
    fn clone(&self) -> SingletonHolder<T> {
        SingletonHolder {
            inner: self.inner.clone(),
        }
    }
}

#[macro_export]
macro_rules! declare_singleton {
    ($name:ident, $t:ty, $init:expr) => {
        fn $name() -> $crate::singleton::SingletonHolder<$t> {
            static mut SINGLETON: *const $crate::singleton::SingletonHolder<$t> =
                0 as *const $crate::singleton::SingletonHolder<$t>;
            static ONCE: ::std::sync::Once = ::std::sync::ONCE_INIT;

            unsafe {
                ONCE.call_once(|| {
                    let singleton = $crate::singleton::SingletonHolder::new(::std::sync::Arc::new(
                        ::std::sync::Mutex::new($init),
                    ));

                    // Put it in the heap so it can outlive this call
                    SINGLETON = ::std::mem::transmute(Box::new(singleton));

                    // Make sure to free heap memory at exit
        					/* This doesn't exist in stable 1.0, so we will just leak it!
        					rt::at_exit(|| {
        						let singleton: Box<SingletonHolder> = mem::transmute(SINGLETON);

        						// Let's explictly free the memory for this example
        						drop(singleton);

        						// Set it to null again. I hope only one thread can call `at_exit`!
        						SINGLETON = 0 as *const _;
        					});
        					*/        });
                (*SINGLETON).clone()
            }
        }
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn smoke_test() {
        declare_singleton!(simple_singleton, u32, 0);
        let simple = simple_singleton();
        match simple.lock() {
            Ok(_) => {}
            Err(_) => {}
        };
    }
}
