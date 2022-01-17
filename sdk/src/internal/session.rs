use crate::errors::{Error, Result};
use crate::internal::client_table::TableServiceChannelPool;
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct Session {
    pub(crate) id: String,

    pub(crate) can_pooled: bool,

    #[derivative(Debug = "ignore")]
    on_drop_callbacks: Vec<Box<dyn FnOnce(&mut Self) + Send + Sync>>,

    #[derivative(Debug = "ignore")]
    channel_pool: TableServiceChannelPool,
}

impl Session {
    pub(crate) fn new(id: String, channel_pool: TableServiceChannelPool) -> Self {
        return Self {
            id,
            can_pooled: true,
            on_drop_callbacks: Vec::new(),
            channel_pool,
        };
    }

    pub(crate) fn handle_error<T>(&mut self, res: Result<T>) -> Result<T> {
        if let Err(Error::YdbOperation(err)) = &res {
            use ydb_protobuf::generated::ydb::status_ids::StatusCode;
            if let Some(status) = StatusCode::from_i32(err.operation_status) {
                if status == StatusCode::BadSession || status == StatusCode::SessionExpired {
                    self.can_pooled = false;
                }
            }
        };
        return res;
    }

    #[allow(dead_code)]
    pub(crate) fn on_drop(&mut self, f: Box<dyn FnOnce(&mut Self) + Send + Sync>) {
        self.on_drop_callbacks.push(f)
    }

    pub(crate) fn clone_without_ondrop(&self) -> Self {
        return Self {
            id: self.id.clone(),
            can_pooled: self.can_pooled,
            on_drop_callbacks: Vec::new(),
            channel_pool: self.channel_pool.clone(),
        };
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        println!("drop session: {}", &self.id);
        while let Some(on_drop) = self.on_drop_callbacks.pop() {
            on_drop(self)
        }
    }
}
