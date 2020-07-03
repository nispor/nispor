use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use nispor::get_state as _get_state;

create_exception!(nispor, NisporError, pyo3::exceptions::Exception);

#[pyfunction]
fn get_state() -> PyResult<String> {
    match _get_state() {
        Ok(state) => Ok(serde_json::to_string_pretty(&state).unwrap()),
        Err(e) => Err(NisporError::py_err(format!("{}", e))),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn nispor(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(get_state))?;
    Ok(())
}
