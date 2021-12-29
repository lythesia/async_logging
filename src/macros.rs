#[macro_export]
macro_rules! log {
    ($n:expr, $log:expr, $fmt:expr, $($msg:expr),* $(,)?) => {{
        let dt = $crate::now();
        let tn = $crate::current_thread_name();
        let s = format!($fmt, $($msg),*);
        $log.log($n, format!("{} [{}] [{}] |{}:{}| {}\n", dt, $n, tn, file!(), line!(), s))
    }};
    ($n:expr, $log:expr, $fmt:expr $(,)?) => {{
        let dt = $crate::now();
        let tn = $crate::current_thread_name();
        let s = format!($fmt);
        $log.log($n, format!("{} [{}] [{}] |{}:{}| {}\n", dt, $n, tn, file!(), line!(), s))
    }};
}

#[macro_export]
macro_rules! error {
    ($log:expr, $fmt:expr, $($msg:expr),* $(,)?) => { $crate::log![$crate::Level::Error, $log, $fmt, $($msg),*] };
    ($log:expr, $fmt:expr $(,)?) => { $crate::log![$crate::Level::Error, $log, $fmt] };
}

#[macro_export]
macro_rules! warn {
    ($log:expr, $fmt:expr, $($msg:expr),* $(,)?) => { $crate::log![$crate::Level::Warn, $log, $fmt, $($msg),*] };
    ($log:expr, $fmt:expr $(,)?) => { $crate::log![$crate::Level::Warn, $log, $fmt] };
}

#[macro_export]
macro_rules! info {
    ($log:expr, $fmt:expr, $($msg:expr),* $(,)?) => { $crate::log![$crate::Level::Info, $log, $fmt, $($msg),*] };
    ($log:expr, $fmt:expr $(,)?) => { $crate::log![$crate::Level::Info, $log, $fmt] };
}

#[macro_export]
macro_rules! debug {
    ($log:expr, $fmt:expr, $($msg:expr),* $(,)?) => { $crate::log![$crate::Level::Debug, $log, $fmt, $($msg),*] };
    ($log:expr, $fmt:expr $(,)?) => { $crate::log![$crate::Level::Debug, $log, $fmt] };
}

#[macro_export]
macro_rules! trace {
    ($log:expr, $fmt:expr, $($msg:expr),* $(,)?) => { $crate::log![$crate::Level::Trace, $log, $fmt, $($msg),*] };
    ($log:expr, $fmt:expr $(,)?) => { $crate::log![$crate::Level::Trace, $log, $fmt] };
}
