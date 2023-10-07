#[macro_export]
macro_rules! get_single {
    ($q:expr) => {
        match $q.get_single() {
            Ok(m) => m,
            _ => return,
        }
    };
}

#[macro_export]
macro_rules! get_some {
    ($q:expr) => {
        match $q {
            Some(m) => m,
            _ => return,
        }
    };
}
