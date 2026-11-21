use std::future::Future;
use std::sync::OnceLock;

pub(crate) const JS_THREAD_STACK_SIZE: usize = 8 * 1024 * 1024;

static JS_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    JS_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("fjs-js")
            .thread_stack_size(JS_THREAD_STACK_SIZE)
            .worker_threads(2)
            .on_thread_start(raise_thread_qos)
            .build()
            .expect("failed to build fjs JavaScript executor")
    })
}

/// Raises the calling thread's Quality-of-Service to `USER_INITIATED` on Apple
/// platforms. Run on every `fjs-js` worker via tokio's `on_thread_start`.
///
/// iOS heavily throttles/coalesces timer wakeups for threads below this QoS:
/// `tokio::time` timers on a default-QoS worker can be delayed by *seconds*
/// (the worker only wakes on socket I/O, never at the timer deadline), which
/// stalls all detached async (`setTimeout`, the response leg of `fetch`).
/// Promoting the JS workers to `USER_INITIATED` keeps the time driver prompt.
/// No-op off Apple.
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn raise_thread_qos() {
    // `QOS_CLASS_USER_INITIATED` from <sys/qos.h> (qos_class_t is a C
    // `unsigned int`). Set via the stable `pthread_set_qos_class_self_np`.
    const QOS_CLASS_USER_INITIATED: core::ffi::c_uint = 0x19;
    unsafe extern "C" {
        fn pthread_set_qos_class_self_np(
            qos_class: core::ffi::c_uint,
            relative_priority: core::ffi::c_int,
        ) -> core::ffi::c_int;
    }
    // SAFETY: FFI call with a valid QoS constant; it only adjusts the calling
    // thread's scheduling class and has no other effects.
    unsafe {
        pthread_set_qos_class_self_np(QOS_CLASS_USER_INITIATED, 0);
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn raise_thread_qos() {}

pub(crate) async fn run_js<F, R>(future: F) -> R
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    runtime()
        .spawn(future)
        .await
        .expect("fjs JavaScript executor task panicked")
}

pub(crate) fn spawn_js<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    runtime().spawn(future)
}
