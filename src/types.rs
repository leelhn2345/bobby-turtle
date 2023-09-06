use teloxide::dispatching::UpdateHandler;

pub type DpHandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub type DpHandler = UpdateHandler<Box<dyn std::error::Error + Send + Sync>>;
