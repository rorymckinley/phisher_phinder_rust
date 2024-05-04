use crate::errors::AppError;
use crate::service_configuration::Configuration;

pub fn execute_command<T>(_config: &T) -> Result<String, AppError>
    where T: Configuration {
    //TODO Unsure how to test error conditions here
    let config_path_buf = confy::get_configuration_file_path("phisher_eagle", None)?;
    let path = config_path_buf.as_path().to_str().unwrap();

    Ok(path.into())
}
