
## 补充债务声明（40号审计后）
- **DEBT-TEST-001**: E2E测试使用Mock Yjs/LevelDB（非真实npm包集成测试）
  - 位置: `tests/p2p/sprint6-integration.e2e.js:11-27`
  - 影响: 无法验证真实Yjs GC、LevelDB文件锁等行为差异
  - 清偿计划: Sprint7替换为真实集成测试（Docker环境）
