/**
 * HNSW (Hierarchical Navigable Small World) 核心实现
 * 
 * 特性：
 * - 纯 JavaScript 实现（无外部依赖，兼容 Termux）
 * - 支持汉明距离（SimHash-64 原生支持）
 * - 支持 L2/Cosine 距离（dense vector）
 * - 内存优化：TypedArray + 对象池
 * - 分层导航图：M=16, efConstruction=200, efSearch=64
 * 
 * 参考：Malkov & Yashunin, "Efficient and robust approximate nearest 
 * neighbor search using Hierarchical Navigable Small World graphs" (2016)
 */

const { hammingDistance, l2Distance, cosineDistance } = require('./distance');

// HNSW 默认配置
const DEFAULT_CONFIG = {
  M: 16,                    // 每层最大连接数
  efConstruction: 200,      // 构建时的搜索深度
  efSearch: 64,             // 搜索时的搜索深度
  maxLevel: 16,             // 最大层数
  distanceMetric: 'hamming', // 'hamming' | 'l2' | 'cosine'
  maxElements: 100000       // 最大元素数（内存预分配）
};

/**
 * 优先队列（最小堆）- 用于贪心搜索
 */
class PriorityQueue {
  constructor(comparator = (a, b) => a.distance - b.distance) {
    this.heap = [];
    this.comparator = comparator;
  }
  
  push(item) {
    this.heap.push(item);
    this._siftUp(this.heap.length - 1);
  }
  
  pop() {
    if (this.heap.length === 0) return null;
    if (this.heap.length === 1) return this.heap.pop();
    
    const top = this.heap[0];
    this.heap[0] = this.heap.pop();
    this._siftDown(0);
    return top;
  }
  
  peek() {
    return this.heap.length > 0 ? this.heap[0] : null;
  }
  
  size() {
    return this.heap.length;
  }
  
  isEmpty() {
    return this.heap.length === 0;
  }
  
  toArray() {
    return [...this.heap];
  }
  
  _siftUp(index) {
    const item = this.heap[index];
    while (index > 0) {
      const parent = Math.floor((index - 1) / 2);
      if (this.comparator(item, this.heap[parent]) >= 0) break;
      this.heap[index] = this.heap[parent];
      index = parent;
    }
    this.heap[index] = item;
  }
  
  _siftDown(index) {
    const item = this.heap[index];
    const len = this.heap.length;
    
    while (true) {
      let min = index;
      const left = 2 * index + 1;
      const right = 2 * index + 2;
      
      if (left < len && this.comparator(this.heap[left], this.heap[min]) < 0) {
        min = left;
      }
      if (right < len && this.comparator(this.heap[right], this.heap[min]) < 0) {
        min = right;
      }
      if (min === index) break;
      
      this.heap[index] = this.heap[min];
      index = min;
    }
    this.heap[index] = item;
  }
}

/**
 * HNSW 节点
 */
class HNSWNode {
  constructor(id, vector, level) {
    this.id = id;
    this.vector = vector;           // Float32Array 或 bigint (SimHash)
    this.level = level;             // 节点所在最高层
    this.connections = [];          // 每层连接的节点ID [level][neighborId]
    
    // 预分配连接数组
    for (let i = 0; i <= level; i++) {
      this.connections[i] = [];
    }
  }
}

/**
 * HNSW 索引主类
 */
class HNSWIndex {
  /**
   * @param {Object} config - 配置选项
   */
  constructor(config = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.nodes = new Map();         // id -> HNSWNode
    this.entryPoint = null;         // 入口节点
    this.maxLevel = 0;              // 当前最大层数
    this.elementCount = 0;          // 当前元素数
    this.deletedCount = 0;          // 已删除标记数
    
    // 距离函数选择
    this.distanceFn = this._getDistanceFunction();
    
    // 层概率因子
    this.levelMult = 1 / Math.log(this.config.M);
  }
  
  /**
   * 获取距离函数
   */
  _getDistanceFunction() {
    switch (this.config.distanceMetric) {
      case 'hamming':
        return hammingDistance;
      case 'l2':
        return l2Distance;
      case 'cosine':
        return cosineDistance;
      default:
        return hammingDistance;
    }
  }
  
  /**
   * 计算距离
   */
  _distance(id1, id2) {
    const node1 = this.nodes.get(id1);
    const node2 = this.nodes.get(id2);
    if (!node1 || !node2) return Infinity;
    return this.distanceFn(node1.vector, node2.vector);
  }
  
  /**
   * 计算查询向量与节点的距离
   */
  _distanceToQuery(queryVector, nodeId) {
    const node = this.nodes.get(nodeId);
    if (!node) return Infinity;
    return this.distanceFn(queryVector, node.vector);
  }
  
  /**
   * 随机生成层数（指数分布）
   */
  _randomLevel() {
    let level = 0;
    const r = Math.random();
    while (r < Math.exp(-level / this.levelMult) && level < this.config.maxLevel) {
      level++;
    }
    return level;
  }
  
  /**
   * 贪心搜索最近邻（单层）
   * @param {Float32Array|bigint} queryVector - 查询向量
   * @param {number} entryId - 入口节点ID
   * @param {number} level - 搜索层
   * @returns {Object} - { id, distance }
   */
  _searchLayer(queryVector, entryId, level) {
    let currentId = entryId;
    let currentDist = this._distanceToQuery(queryVector, currentId);
    let changed = true;
    
    while (changed) {
      changed = false;
      const node = this.nodes.get(currentId);
      if (!node || !node.connections[level]) break;
      
      for (const neighborId of node.connections[level]) {
        const neighborDist = this._distanceToQuery(queryVector, neighborId);
        if (neighborDist < currentDist) {
          currentId = neighborId;
          currentDist = neighborDist;
          changed = true;
        }
      }
    }
    
    return { id: currentId, distance: currentDist };
  }
  
  /**
   * 多候选搜索（ef 控制）
   * @param {Float32Array|bigint} queryVector 
   * @param {number} entryId 
   * @param {number} ef - 搜索宽度
   * @param {number} level - 层
   * @returns {Array} - 最近邻列表 [{id, distance}]
   */
  _searchLayerEf(queryVector, entryId, ef, level) {
    const visited = new Set([entryId]);
    const candidates = new PriorityQueue((a, b) => b.distance - a.distance); // 最大堆
    const results = new PriorityQueue((a, b) => b.distance - a.distance);    // 最大堆
    
    const entryDist = this._distanceToQuery(queryVector, entryId);
    candidates.push({ id: entryId, distance: entryDist });
    results.push({ id: entryId, distance: entryDist });
    
    while (!candidates.isEmpty()) {
      const current = candidates.pop();
      const worstResult = results.peek();
      
      if (current.distance > worstResult.distance) break;
      
      const node = this.nodes.get(current.id);
      if (!node || !node.connections[level]) continue;
      
      for (const neighborId of node.connections[level]) {
        if (visited.has(neighborId)) continue;
        visited.add(neighborId);
        
        const dist = this._distanceToQuery(queryVector, neighborId);
        const worst = results.peek();
        
        if (results.size() < ef || dist < worst.distance) {
          candidates.push({ id: neighborId, distance: dist });
          results.push({ id: neighborId, distance: dist });
          
          if (results.size() > ef) {
            results.pop();
          }
        }
      }
    }
    
    return results.toArray().sort((a, b) => a.distance - b.distance);
  }
  
  /**
   * 选择最近邻（启发式，保持图连通性）
   * @param {Float32Array|bigint} vector 
   * @param {Array} candidates - 候选列表 [{id, distance}]
   * @param {number} M - 最大连接数
   * @returns {Array} - 选中的邻居ID
   */
  _selectNeighbors(vector, candidates, M) {
    if (candidates.length <= M) {
      return candidates.map(c => c.id);
    }
    
    // 多样性启发式：在距离和多样性之间平衡
    // 策略：优先选择距离近的，但跳过与已选邻居过于相似的
    const selected = [];
    const selectedVectors = [];
    
    // 按距离排序（已由调用者保证）
    for (const candidate of candidates) {
      if (selected.length >= M) break;
      
      // 获取候选节点向量
      const candidateNode = this.nodes.get(candidate.id);
      if (!candidateNode) continue;
      
      // 检查与已选邻居的多样性（最小距离阈值）
      let isDiverse = true;
      for (const sv of selectedVectors) {
        const dist = this._distance(candidateNode.vector, sv);
        // 如果与已选邻居太相似（距离太小），则跳过
        if (dist < candidate.distance * 0.3) {
          isDiverse = false;
          break;
        }
      }
      
      if (isDiverse) {
        selected.push(candidate.id);
        selectedVectors.push(candidateNode.vector);
      }
    }
    
    // 如果多样性选择不够M个，补充最近的
    if (selected.length < M) {
      for (const candidate of candidates) {
        if (selected.length >= M) break;
        if (!selected.includes(candidate.id)) {
          selected.push(candidate.id);
        }
      }
    }
    
    return selected;
  }
  
  /**
   * 插入向量
   * @param {number} id - 唯一标识
   * @param {Float32Array|bigint} vector - 向量
   * @returns {boolean}
   */
  insert(id, vector) {
    if (this.nodes.has(id)) {
      throw new Error(`Node with id ${id} already exists`);
    }
    
    const level = this._randomLevel();
    const node = new HNSWNode(id, vector, level);
    this.nodes.set(id, node);
    this.elementCount++;
    
    // 第一个节点作为入口
    if (!this.entryPoint) {
      this.entryPoint = id;
      this.maxLevel = level;
      return true;
    }
    
    // 从最高层开始搜索
    let currentId = this.entryPoint;
    const efConstruction = this.config.efConstruction;
    
    // 如果新节点层数高于当前最大层，更新入口
    if (level > this.maxLevel) {
      for (let i = this.maxLevel + 1; i <= level; i++) {
        this.entryPoint = id;
      }
      this.maxLevel = level;
    }
    
    // 贪心搜索入口点
    for (let i = this.maxLevel; i > level; i--) {
      const result = this._searchLayer(vector, currentId, i);
      currentId = result.id;
    }
    
    // 逐层插入连接
    for (let i = Math.min(level, this.maxLevel); i >= 0; i--) {
      const neighbors = this._searchLayerEf(vector, currentId, efConstruction, i);
      const selectedNeighbors = this._selectNeighbors(vector, neighbors, this.config.M);
      
      node.connections[i] = selectedNeighbors;
      
      // 双向连接
      for (const neighborId of selectedNeighbors) {
        const neighbor = this.nodes.get(neighborId);
        if (neighbor && neighbor.connections[i]) {
          neighbor.connections[i].push(id);
          
          // 如果超出M限制，裁剪
          if (neighbor.connections[i].length > this.config.M * 2) {
            neighbor.connections[i] = this._pruneConnections(neighbor, i);
          }
        }
      }
      
      currentId = neighbors[0]?.id || currentId;
    }
    
    return true;
  }
  
  /**
   * 裁剪连接（保持最近M个）
   */
  _pruneConnections(node, level) {
    const connections = node.connections[level];
    if (connections.length <= this.config.M) return connections;
    
    // 计算到所有连接的距离
    const distances = connections.map(id => ({
      id,
      distance: this._distance(node.id, id)
    }));
    
    distances.sort((a, b) => a.distance - b.distance);
    return distances.slice(0, this.config.M).map(d => d.id);
  }
  
  /**
   * 搜索最近邻
   * @param {Float32Array|bigint} queryVector - 查询向量
   * @param {number} k - 返回数量
   * @param {Object} options - { efSearch }
   * @returns {Array} - [{id, distance}]
   */
  search(queryVector, k = 10, options = {}) {
    if (this.entryPoint === null || this.elementCount === 0) {
      return [];
    }
    
    // 单元素特殊情况
    if (this.elementCount === 1) {
      const entry = this.nodes.get(this.entryPoint);
      if (entry && !entry.deleted) {
        return [{
          id: this.entryPoint,
          distance: this._distanceToQuery(queryVector, this.entryPoint)
        }];
      }
      return [];
    }
    
    const efSearch = options.efSearch || this.config.efSearch;
    let currentId = this.entryPoint;
    
    // 从最高层贪心下降到第1层
    for (let i = this.maxLevel; i >= 1; i--) {
      const result = this._searchLayer(queryVector, currentId, i);
      currentId = result.id;
    }
    
    // 第0层使用 ef 搜索
    const results = this._searchLayerEf(queryVector, currentId, Math.max(efSearch, k), 0);
    
    return results.slice(0, k);
  }
  
  /**
   * 删除节点（软删除标记）
   * @param {number} id 
   * @returns {boolean}
   */
  delete(id) {
    const node = this.nodes.get(id);
    if (!node) return false;
    
    // 标记删除（实际从图中移除较复杂，这里简化为标记）
    node.deleted = true;
    this.deletedCount++;
    
    // 如果删除的是入口点，需要重新选择
    if (this.entryPoint === id) {
      // 简单策略：找任意非删除节点
      for (const [nid, n] of this.nodes) {
        if (!n.deleted) {
          this.entryPoint = nid;
          break;
        }
      }
      if (this.entryPoint === id) {
        this.entryPoint = null;
      }
    }
    
    return true;
  }
  
  /**
   * 获取节点
   * @param {number} id 
   * @returns {HNSWNode|null}
   */
  getNode(id) {
    return this.nodes.get(id) || null;
  }
  
  /**
   * 获取统计信息
   */
  getStats() {
    let totalConnections = 0;
    for (const node of this.nodes.values()) {
      for (const conns of node.connections) {
        totalConnections += conns?.length || 0;
      }
    }
    
    return {
      elementCount: this.elementCount,
      deletedCount: this.deletedCount,
      activeCount: this.elementCount - this.deletedCount,
      maxLevel: this.maxLevel,
      entryPoint: this.entryPoint,
      avgConnections: this.elementCount > 0 ? totalConnections / this.elementCount : 0,
      config: { ...this.config }
    };
  }
  
  /**
   * 序列化为 JSON
   */
  toJSON() {
    const nodes = [];
    for (const [id, node] of this.nodes) {
      nodes.push({
        id,
        vector: typeof node.vector === 'bigint' ? node.vector.toString() : Array.from(node.vector),
        level: node.level,
        connections: node.connections,
        deleted: node.deleted || false
      });
    }
    
    return {
      config: this.config,
      maxLevel: this.maxLevel,
      entryPoint: this.entryPoint,
      elementCount: this.elementCount,
      deletedCount: this.deletedCount,
      nodes
    };
  }
  
  /**
   * 从 JSON 恢复
   * @param {Object} json 
   * @returns {HNSWIndex}
   */
  static fromJSON(json) {
    const index = new HNSWIndex(json.config);
    index.maxLevel = json.maxLevel;
    index.entryPoint = json.entryPoint;
    index.elementCount = json.elementCount;
    index.deletedCount = json.deletedCount || 0;
    
    for (const n of json.nodes) {
      const vector = typeof n.vector === 'string' ? BigInt(n.vector) : new Float32Array(n.vector);
      const node = new HNSWNode(n.id, vector, n.level);
      node.connections = n.connections;
      node.deleted = n.deleted || false;
      index.nodes.set(n.id, node);
    }
    
    return index;
  }
  
  /**
   * 清空索引
   */
  clear() {
    this.nodes.clear();
    this.entryPoint = null;
    this.maxLevel = 0;
    this.elementCount = 0;
    this.deletedCount = 0;
  }
}

module.exports = {
  HNSWIndex,
  HNSWNode,
  PriorityQueue
};
