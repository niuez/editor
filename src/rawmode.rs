use libc::{winsize, STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ};
use termios::{tcsetattr, Termios, TCSAFLUSH};

pub struct RawMode {
    orig_term: Termios,
}

impl RawMode {
    pub fn enable_raw_mode() -> std::io::Result<Self> {
        use termios::*;
        let mut term = Termios::from_fd(STDIN_FILENO)?;
        let mode = Self { orig_term: term };

        term.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        term.c_oflag &= !OPOST;
        term.c_cflag |= CS8;
        term.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        term.c_cc[VMIN] = 0;
        term.c_cc[VTIME] = 1;

        tcsetattr(STDIN_FILENO, TCSAFLUSH, &term)?;
        Ok(mode)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.orig_term).expect("Failed to drop RawMode")
    }
}
