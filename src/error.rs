use std::error::Error;

#[derive(Debug, Clone)]
pub struct OpenstackError {
    pub details: String,
}

impl std::fmt::Display for OpenstackError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.details)
    }
}

impl OpenstackError {
    pub fn new(msg: &str) -> OpenstackError {
        OpenstackError {
            details: msg.to_string(),
        }
    }
}

impl std::error::Error for OpenstackError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<curl::Error> for OpenstackError {
    fn from(err: curl::Error) -> Self {
        OpenstackError::new(err.description())
    }
}

impl From<curl::easy::ReadError> for OpenstackError {
    fn from(err: curl::easy::ReadError) -> Self {
        OpenstackError::new(&format!("{:?}", err))
    }
}

impl From<curl::easy::WriteError> for OpenstackError {
    fn from(err: curl::easy::WriteError) -> Self {
        OpenstackError::new(&format!("{:?}", err))
    }
}

impl From<curl::MultiError> for OpenstackError {
    fn from(err: curl::MultiError) -> Self {
        OpenstackError::new(&format!("{:?}", err))
    }
}

impl From<std::io::Error> for OpenstackError {
    fn from(err: std::io::Error) -> Self {
        OpenstackError::new(err.description())
    }
}

impl From<handlebars::TemplateRenderError> for OpenstackError{
    fn from(err: handlebars::TemplateRenderError) -> Self {
        OpenstackError::new(err.description())
    }
}