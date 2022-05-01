#[macro_export]
macro_rules! async_ok {
    ($result:expr) => {
        match $result.await {
            Ok(value) => {
                tracing::debug!("res: {:?}", value);
                Ok(value)
            }
            Err(err) => {
                tracing::error!("error {:?}", err);
                Err(err)
            }
        }
    };
}

#[macro_export]
macro_rules! sync_ok {
    ($result:expr) => {
        match $result {
            Ok(value) => {
                tracing::info!("res: {:?}", value);
                Ok(value)
            }
            Err(err) => {
                tracing::error!("error {:?}", err);
                Err(err)
            }
        }
    };
}
