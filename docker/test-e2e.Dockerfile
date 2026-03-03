# 真实Yjs+LevelDB E2E测试环境
# DEBT-TEST-001清偿: Docker隔离运行真实npm包测试

FROM node:18-alpine

# 安装必要依赖
RUN apk add --no-cache bash curl

WORKDIR /app

# 复制package文件并安装依赖
COPY package*.json ./
RUN npm ci --only=production && \
    npm install yjs@^13.6.0 level@^8.0.1

# 创建数据目录
RUN mkdir -p /app/data

# 复制测试文件
COPY tests/p2p/real-yjs-level.e2e.js ./tests/

# 设置环境变量
ENV NODE_ENV=test
ENV TEST_DB_PATH=/app/data/test.db

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD node --version || exit 1

# 测试执行入口
CMD ["node", "tests/real-yjs-level.e2e.js"]
