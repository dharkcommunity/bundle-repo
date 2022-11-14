pub fn validate_resource_name(resource: &str) -> Result<(), String> {
    if resource.len() < 2 || resource.len() > 32 {
        return Err("Resource names must be between 2-32 characters in length".to_string());
    }
    if resource.chars().any(|c| !c.is_alphanumeric()) {
        return Err("Resource names must only contain alphanumeric characters".to_string());
    }
    Ok(())
}
