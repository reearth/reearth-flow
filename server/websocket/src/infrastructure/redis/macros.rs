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

macro_rules! doc_instance_key {
    ($doc_id:expr) => {
        format!("doc:instance:{}", $doc_id)
    };
}

macro_rules! read_lock_key {
    ($doc_id:expr) => {
        format!("read:lock:{}", $doc_id)
    };
}

pub(crate) const RELEASE_LOCK_SCRIPT: &str = r#"
if redis.call('get', KEYS[1]) == ARGV[1] then
    return redis.call('del', KEYS[1])
else
    return 0
end
"#;

macro_rules! exec_script {
    ($pool:expr, $script:expr, keys: [$($key:expr),*], args: [$($arg:expr),*] => $ret_ty:ty) => {{
        let mut conn = $pool.get().await?;
        let script = redis::Script::new($script);
        let result: $ret_ty = script
            $(.key($key))*
            $(.arg($arg))*
            .invoke_async(&mut *conn)
            .await?;
        result
    }};
}

macro_rules! acquire_lock_with_ttl {
    ($pool:expr, $key:expr, $value:expr, $ttl:expr) => {{
        let mut conn = $pool.get().await?;
        let result: Option<String> = redis::cmd("SET")
            .arg($key)
            .arg($value)
            .arg("NX")
            .arg("EX")
            .arg($ttl)
            .query_async(&mut *conn)
            .await?;
        result.is_some()
    }};
}

pub(crate) use {
    acquire_lock_with_ttl, doc_instance_key, exec_script, instances_key, lock_key, read_lock_key,
    redis_cmd, stream_key, timestamp_millis, timestamp_secs,
};
