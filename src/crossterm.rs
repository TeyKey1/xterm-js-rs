//! Implements xtermjs as crossterm backend
//!
//! Code partially taken from https://github.com/rrbutani/xterm-js-sys by [rrbutani](https://github.com/rrbutani) MIT licensed (https://github.com/rrbutani/xterm-js-sys/blob/main/LICENSE)

//! Supporting types for a xterm.js-backed backend for [crossterm].
//!
//! [crossterm]: docs.rs/crossterm
use std::cell::Cell;
use std::fmt::{self, Debug};
use std::io::{Error as IoError, ErrorKind, Result as IoResult, Write};
use std::ops::Deref;
use std::rc::Rc;

use super::xterm::Terminal;

/// Wrapper for the [xterm.js terminal](Terminal) for use with [crossterm].
///
/// [crossterm]: docs.rs/crossterm
pub struct XtermJsCrosstermBackend {
    /// The xterm.js terminal that this struct instance wraps.
    pub terminal: Rc<Terminal>,
    /// Internal buffer for data to write to the terminal.
    ///
    /// This lets us make one big call to [`Terminal::write`] with a batch of
    /// commands rather than many small calls.
    buffer: Cell<Vec<u8>>,
}

impl Debug for XtermJsCrosstermBackend {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct(core::any::type_name::<Self>()).finish()
    }
}

impl Deref for XtermJsCrosstermBackend {
    type Target = Terminal;

    fn deref(&self) -> &Terminal {
        //! This will flush the internal buffer before providing the reference
        //! to make sure that the order of operations is preserved.
        self.flush_immutable().unwrap();
        &*self.terminal
    }
}

impl Drop for XtermJsCrosstermBackend {
    fn drop(&mut self) {
        self.flush().unwrap()
    }
}

impl XtermJsCrosstermBackend {
    /// Constructor for the wrapper type.
    #[must_use]
    pub fn new(terminal: Rc<Terminal>) -> Self {
        Self {
            terminal,
            buffer: Cell::new(Vec::new()),
        }
    }

    /// A version of [`flush`](Write::flush) that takes an immutable reference
    /// instead of a mutable one.
    ///
    /// This exists because we want to flush the buffer in the [`Deref`] impl.
    #[inline]
    fn flush_immutable(&self) -> IoResult<()> {
        // Can't call `self.buffer.flush()` here but since that's just a Vec,
        // it's probably fine.

        let s = String::from_utf8(self.buffer.replace(Vec::new()))
            .map_err(|e| IoError::new(ErrorKind::Other, e))?;
        self.terminal.write(&s);

        Ok(())
    }
}

impl Write for XtermJsCrosstermBackend {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.buffer.get_mut().write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.buffer.get_mut().flush()?;

        self.flush_immutable()
    }
}

impl From<Rc<Terminal>> for XtermJsCrosstermBackend {
    fn from(terminal: Rc<Terminal>) -> Self {
        Self::new(terminal)
    }
}
