use ::failure::Fail;
use app_dirs::AppDirsError;
use config::ConfigError;

#[derive(Fail, Debug)]
pub enum EnvyError {
    #[fail(display = "{}", _0)]
    AppDirsError(#[cause] AppDirsError),
    #[fail(display = "Cannot load the config file: {}", _0)]
    ConfigError(#[cause] ConfigError),
    #[fail(display = "{}", _0)]
    Io(#[cause] ::std::io::Error),
    #[fail(display = "{}", _0)]
    InvalidShell(String),
}

impl From<AppDirsError> for EnvyError {
    fn from(e: AppDirsError) -> Self {
        EnvyError::AppDirsError(e)
    }
}

impl From<ConfigError> for EnvyError {
    fn from(e: ConfigError) -> Self {
        EnvyError::ConfigError(e)
    }
}

impl From<std::io::Error> for EnvyError {
    fn from(e: ::std::io::Error) -> Self {
        EnvyError::Io(e)
    }
}
