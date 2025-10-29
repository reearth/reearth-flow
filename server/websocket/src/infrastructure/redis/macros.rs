macro_rules! redis_cmd {
    (
        $(#[$meta:meta])*
        pub async fn $fn_name:ident(&self, $($arg:ident: $arg_ty:ty),*) -> Result<$ret_ty:ty> {
            $cmd:expr
            $(, arg($($cmd_arg:expr),+))*
        }
    ) => {
        $(#[$meta])*
        pub async fn $fn_name(&self, $($arg: $arg_ty),*) -> Result<$ret_ty> {
            let mut conn = self.pool.get().await?;
            let result: $ret_ty = $cmd
                $(
                    $(.arg($cmd_arg))+
                )*
                .query_async(&mut *conn)
                .await?;
            Ok(result)
        }
    };
}

macro_rules! timestamp_millis {
    () => {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    };
}

macro_rules! timestamp_secs {
    () => {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };
}

macro_rules! stream_key {
    ($doc_id:expr) => {
        format!("yjs:stream:{}", $doc_id)
    };
}

macro_rules! instances_key {
    ($doc_id:expr) => {
        format!("doc:instances:{}", $doc_id)
    };
}

macro_rules! lock_key {
    ($doc_id:expr) => {
        format!("lock:doc:{}", $doc_id)
    };
}

pub(crate) use {instances_key, lock_key, redis_cmd, stream_key, timestamp_millis, timestamp_secs};
