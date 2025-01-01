use error_iter::ErrorIter as _;
use log::error;

pub(crate) fn log_error<E: std::error::Error + 'static>(err: E) {
    error!("{err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
