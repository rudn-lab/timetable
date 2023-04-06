use std::collections::HashMap;

#[derive(Debug)]
pub enum RouteError {
    InvalidQuery(String),
}

pub fn parse_url_params(query: &str) -> Result<HashMap<String, String>, RouteError> {
    query
        .split('&')
        .map(|param| -> Result<(String, String), RouteError> {
            let mut params = param.split('=');
            Ok((
                params
                    .next()
                    .ok_or(RouteError::InvalidQuery(param.to_string()))?
                    .to_string(),
                params
                    .next()
                    .ok_or(RouteError::InvalidQuery(param.to_string()))?
                    .to_string(),
            ))
        })
        .collect::<Result<HashMap<String, String>, RouteError>>()
}
