use handlebars::{TemplateError, RenderError};
use pandoc::PandocError;
use std::{error, fmt, io, result};

#[derive(Debug)]
pub enum Error {
    HandlebarsTemplate(TemplateError),
    HandlebarsRender(RenderError),
    Io(io::Error),
    Pandoc(PandocError),
    UnrecognisedExtension(String),
}

pub type Result<T> = result::Result<T, Error>;

impl From<TemplateError> for Error {
    fn from(err: TemplateError) -> Error {
        Error::HandlebarsTemplate(err)
    }
}

impl From<RenderError> for Error {
    fn from(err: RenderError) -> Error {
        Error::HandlebarsRender(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<PandocError> for Error {
    fn from(err: PandocError) -> Error {
        Error::Pandoc(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::HandlebarsTemplate(ref e) => e.fmt(f),
            Error::HandlebarsRender(ref e) => e.fmt(f),
            Error::Io(ref e) => e.fmt(f),
            Error::Pandoc(_) => write!(f, "Pandoc error."),
            Error::UnrecognisedExtension(ref e) => {
                write!(f, "Unrecognised file type: {}", e)
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::HandlebarsTemplate(ref e) => e.description(),
            Error::HandlebarsRender(ref e) => e.description(),
            Error::Io(ref e) => e.description(),
            Error::Pandoc(_) => "Pandoc error.",
            Error::UnrecognisedExtension(_) => "Unrecognised file type",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::HandlebarsTemplate(ref e) => Some(e),
            Error::HandlebarsRender(ref e) => Some(e),
            Error::Io(ref e) => Some(e),
            // PandocError does not implement std::error::Error
            Error::Pandoc(_) => None,
            Error::UnrecognisedExtension(_) => None,
        }
    }
}