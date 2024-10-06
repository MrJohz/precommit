use std::{cell::RefCell, fmt::Arguments, io::Write, rc::Rc};

use crate::errors::Error;

pub trait World: Clone {
    type Stdout: Write;
    type Stderr: Write;

    fn output(&self, bytes: &[u8]) -> Result<(), Error>;

    fn check_failed_info(&self, args: Arguments) -> Result<(), Error>;
    fn check_failed(&self, args: Arguments) -> Result<(), Error>;
    fn warning(&self, args: Arguments) -> Result<(), Error>;
    fn error(&self, args: Arguments) -> Result<(), Error>;

    fn stderr_raw_bytes(&self, bytes: &[u8]) -> Result<(), Error>;
}

pub struct WriterWorld<Stdout, Stderr> {
    stdout: Rc<RefCell<Stdout>>,
    stderr: Rc<RefCell<Stderr>>,
}

impl<Stdout, Stderr> Clone for WriterWorld<Stdout, Stderr> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
        }
    }
}

impl<Stdout, Stderr> WriterWorld<Stdout, Stderr> {
    #[inline]
    pub fn new(stdout: Stdout, stderr: Stderr) -> Self {
        WriterWorld {
            stdout: Rc::new(RefCell::new(stdout)),
            stderr: Rc::new(RefCell::new(stderr)),
        }
    }
}

impl<Stdout: Clone, Stderr: Clone> WriterWorld<Stdout, Stderr> {
    pub fn outputs(self) -> (Stdout, Stderr) {
        let WriterWorld { stdout, stderr } = self;
        (
            <std::cell::RefCell<Stdout> as Clone>::clone(&stdout).into_inner(),
            <std::cell::RefCell<Stderr> as Clone>::clone(&stderr).into_inner(),
        )
    }
}

impl<Stdout, Stderr> World for WriterWorld<Stdout, Stderr>
where
    Stdout: Write,
    Stderr: Write,
    WriterWorld<Stdout, Stderr>: Clone,
{
    type Stdout = Stdout;
    type Stderr = Stderr;

    #[inline]
    fn output(&self, bytes: &[u8]) -> Result<(), Error> {
        self.stdout.borrow_mut().write_all(bytes)?;
        Ok(())
    }

    #[inline]
    fn stderr_raw_bytes(&self, bytes: &[u8]) -> Result<(), Error> {
        self.stderr.borrow_mut().write_all(bytes)?;
        Ok(())
    }

    #[inline]
    fn warning(&self, args: Arguments) -> Result<(), Error> {
        self.stderr.borrow_mut().write_all(b"\x1b[0;1;33m")?;
        self.stderr.borrow_mut().write_fmt(args)?;
        self.stderr.borrow_mut().write_all(b"\x1b[0m\n")?;
        Ok(())
    }

    #[inline]
    fn error(&self, args: Arguments) -> Result<(), Error> {
        self.stderr.borrow_mut().write_all(b"\x1b[0;1;31m")?;
        self.stderr.borrow_mut().write_fmt(args)?;
        self.stderr.borrow_mut().write_all(b"\x1b[0m\n")?;
        Ok(())
    }

    #[inline]
    fn check_failed(&self, args: Arguments) -> Result<(), Error> {
        self.stderr.borrow_mut().write_all(b"\x1b[0;31m")?;
        self.stderr.borrow_mut().write_fmt(args)?;
        self.stderr.borrow_mut().write_all(b"\x1b[0m\n")?;
        Ok(())
    }

    #[inline]
    fn check_failed_info(&self, args: Arguments) -> Result<(), Error> {
        self.stderr.borrow_mut().write_all(b"\x1b[0;33m > ")?;
        self.stderr.borrow_mut().write_fmt(args)?;
        self.stderr.borrow_mut().write_all(b"\x1b[0m\n")?;
        Ok(())
    }
}
