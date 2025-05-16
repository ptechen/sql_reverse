#[macro_export]
macro_rules! async_ok {
    ($result:expr) => {
        match $result.await {
            Ok(value) => {
                log::debug!("res: {:?}", value);
                Ok(value)
            }
            Err(err) => {
                log::error!("error {:?}", err);
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
                log::info!("res: {:?}", value);
                Ok(value)
            }
            Err(err) => {
                log::error!("error {:?}", err);
                Err(err)
            }
        }
    };
}
