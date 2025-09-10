#!/bin/bash

# Redis 清理脚本 - 清除旧的 WebSocket 数据以避免格式不兼容问题

set -e

REDIS_URL=${REDIS_URL:-"redis://localhost:6379"}
REDIS_HOST=${REDIS_HOST:-"localhost"}
REDIS_PORT=${REDIS_PORT:-"6379"}
PREFIX=${PREFIX:-"y"}

echo "🧹 正在清理 Redis WebSocket 数据..."
echo "Redis 地址: $REDIS_URL"
echo "前缀: $PREFIX"

# 检查 Redis 是否可用
echo "检查 Redis 连接..."
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" ping

if [ $? -ne 0 ]; then
    echo "❌ 无法连接到 Redis 服务器"
    exit 1
fi

echo "✅ Redis 连接成功"

# 清理 YJS 相关的数据
echo "清理 YJS streams..."
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" --scan --pattern "${PREFIX}:*" | xargs -I {} redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" DEL {}

echo "清理 worker streams..."
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" DEL "${PREFIX}:worker"

echo "清理 consumer groups..."
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" XGROUP DESTROY "${PREFIX}:worker" "${PREFIX}:worker" 2>/dev/null || true

echo "清理文档实例信息..."
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" --scan --pattern "doc:instance:*" | xargs -I {} redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" DEL {}
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" --scan --pattern "doc:instances:*" | xargs -I {} redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" DEL {}

echo "清理锁信息..."
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" --scan --pattern "lock:*" | xargs -I {} redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" DEL {}
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" --scan --pattern "read:lock:*" | xargs -I {} redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" DEL {}

echo "清理 stream 相关数据..."
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" --scan --pattern "yjs:stream:*" | xargs -I {} redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" DEL {}

echo "🎉 Redis 清理完成！"
echo ""
echo "现在可以安全地启动新的 WebSocket 服务器："
echo "  cargo run --bin websocket"
echo ""
echo "或者带认证功能："
echo "  cargo run --bin websocket --features auth"
