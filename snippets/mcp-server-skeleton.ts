/**
 * MCP Server TypeScript骨架
 * 基于 @modelcontextprotocol/sdk
 * 
 * 使用方法：
 * 1. npm init -y
 * 2. npm install @modelcontextprotocol/sdk zod
 * 3. npx tsc --init
 * 4. node dist/mcp-server-skeleton.js
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ListResourcesRequestSchema,
  ReadResourceRequestSchema,
  ListPromptsRequestSchema,
  GetPromptRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";

// ============================================================
// 1. 服务器元数据配置
// ============================================================
const SERVER_INFO = {
  name: "hajimi-example-server",
  version: "1.0.0",
};

// ============================================================
// 2. ServerCapabilities定义
// ============================================================
const SERVER_CAPABILITIES = {
  // 工具能力：暴露可调用的函数
  tools: {
    listChanged: true, // 支持工具列表变更通知
  },
  // 资源能力：暴露可读的数据源
  resources: {
    subscribe: true,   // 支持资源变更订阅
    listChanged: true, // 支持资源列表变更通知
  },
  // 提示词能力：暴露可复用的提示模板
  prompts: {
    listChanged: true, // 支持提示词列表变更通知
  },
  // 日志能力
  logging: {},
};

// ============================================================
// 3. Tool定义
// ============================================================
const TOOLS = [
  {
    name: "echo",
    description: "Echo back the input message",
    inputSchema: {
      type: "object" as const,
      properties: {
        message: {
          type: "string",
          description: "Message to echo back",
        },
      },
      required: ["message"],
    },
  },
  {
    name: "calculate",
    description: "Perform basic arithmetic calculation",
    inputSchema: {
      type: "object" as const,
      properties: {
        operation: {
          type: "string",
          enum: ["add", "subtract", "multiply", "divide"],
          description: "Arithmetic operation to perform",
        },
        a: {
          type: "number",
          description: "First operand",
        },
        b: {
          type: "number",
          description: "Second operand",
        },
      },
      required: ["operation", "a", "b"],
    },
  },
];

// ============================================================
// 4. Resource定义
// ============================================================
const RESOURCES = [
  {
    uri: "config://server.json",
    name: "Server Configuration",
    description: "Current server configuration metadata",
    mimeType: "application/json",
  },
  {
    uri: "status://health",
    name: "Health Status",
    description: "Server health and uptime information",
    mimeType: "application/json",
  },
];

// ============================================================
// 5. Prompt定义
// ============================================================
const PROMPTS = [
  {
    name: "code_review",
    description: "Generate a code review prompt",
    arguments: [
      {
        name: "language",
        description: "Programming language",
        required: true,
      },
      {
        name: "focus",
        description: "Review focus area (security, performance, style)",
        required: false,
      },
    ],
  },
];

// ============================================================
// 6. 服务器实例化
// ============================================================
const server = new Server(
  {
    name: SERVER_INFO.name,
    version: SERVER_INFO.version,
  },
  {
    capabilities: SERVER_CAPABILITIES,
  }
);

// ============================================================
// 7. 请求处理器注册
// ============================================================

/**
 * tools/list - 列出可用工具
 * Client通过此端点发现服务器提供的所有工具
 */
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return { tools: TOOLS };
});

/**
 * tools/call - 调用指定工具
 * Client通过此端点执行工具功能
 */
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "echo": {
        const message = args?.message as string;
        return {
          content: [
            {
              type: "text" as const,
              text: `Echo: ${message}`,
            },
          ],
        };
      }

      case "calculate": {
        const { operation, a, b } = args as {
          operation: string;
          a: number;
          b: number;
        };
        let result: number;

        switch (operation) {
          case "add":
            result = a + b;
            break;
          case "subtract":
            result = a - b;
            break;
          case "multiply":
            result = a * b;
            break;
          case "divide":
            if (b === 0) throw new Error("Division by zero");
            result = a / b;
            break;
          default:
            throw new Error(`Unknown operation: ${operation}`);
        }

        return {
          content: [
            {
              type: "text" as const,
              text: `Result: ${result}`,
            },
          ],
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : "Unknown error";
    return {
      content: [
        {
          type: "text" as const,
          text: `Error: ${errorMessage}`,
        },
      ],
      isError: true,
    };
  }
});

/**
 * resources/list - 列出可用资源
 */
server.setRequestHandler(ListResourcesRequestSchema, async () => {
  return { resources: RESOURCES };
});

/**
 * resources/read - 读取资源内容
 */
server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const { uri } = request.params;

  switch (uri) {
    case "config://server.json": {
      return {
        contents: [
          {
            uri,
            mimeType: "application/json",
            text: JSON.stringify(
              {
                name: SERVER_INFO.name,
                version: SERVER_INFO.version,
                capabilities: SERVER_CAPABILITIES,
                uptime: process.uptime(),
              },
              null,
              2
            ),
          },
        ],
      };
    }

    case "status://health": {
      return {
        contents: [
          {
            uri,
            mimeType: "application/json",
            text: JSON.stringify(
              {
                status: "healthy",
                uptime: process.uptime(),
                timestamp: new Date().toISOString(),
              },
              null,
              2
            ),
          },
        ],
      };
    }

    default:
      throw new Error(`Resource not found: ${uri}`);
  }
});

/**
 * prompts/list - 列出可用提示词
 */
server.setRequestHandler(ListPromptsRequestSchema, async () => {
  return { prompts: PROMPTS };
});

/**
 * prompts/get - 获取提示词内容
 */
server.setRequestHandler(GetPromptRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  if (name === "code_review") {
    const language = args?.language as string;
    const focus = (args?.focus as string) || "general";

    return {
      description: `Code review prompt for ${language}`,
      messages: [
        {
          role: "user" as const,
          content: {
            type: "text" as const,
            text: `Please review the following ${language} code with focus on ${focus}:

1. Identify potential issues or improvements
2. Suggest best practices for ${language}
3. Highlight any security concerns if applicable
4. Provide specific code examples for improvements

Code to review:
[Code will be inserted here]`,
          },
        },
      ],
    };
  }

  throw new Error(`Prompt not found: ${name}`);
});

// ============================================================
// 8. 服务器启动
// ============================================================
async function main() {
  // 使用stdio传输层（适合本地进程通信）
  const transport = new StdioServerTransport();

  // 连接服务器到传输层
  await server.connect(transport);

  // 向stderr输出日志（stdout用于JSON-RPC通信）
  console.error(`MCP Server "${SERVER_INFO.name}" v${SERVER_INFO.version} started`);
  console.error("Waiting for connections...");
}

// 启动服务器
main().catch((error) => {
  console.error("Failed to start server:", error);
  process.exit(1);
});

// ============================================================
// package.json 依赖参考
// ============================================================
/*
{
  "name": "hajimi-mcp-server",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "build": "tsc",
    "start": "node dist/mcp-server-skeleton.js"
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.0",
    "zod": "^3.22.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0"
  }
}
*/

// ============================================================
// tsconfig.json 配置参考
// ============================================================
/*
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "Node16",
    "moduleResolution": "Node16",
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
*/
