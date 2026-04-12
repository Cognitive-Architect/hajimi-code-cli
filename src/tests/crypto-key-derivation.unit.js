/**
 * 密钥派生单元测试 - 验证scryptSync密钥派生
 * 测试: 相同sharedSecret派生相同密钥 | 不同secret派生不同密钥 | AES-GCM加解密
 */
const crypto = require('crypto');
const { DataChannelManager } = require('../src/p2p/datachannel-manager.js');

let passed = 0, failed = 0;

function assert(cond, msg) {
  if (cond) { passed++; console.log(`✓ ${msg}`); }
  else { failed++; console.error(`✗ ${msg}`); }
}

console.log('=== 密钥派生单元测试 ===\n');

// 测试1: 相同sharedSecret派生相同密钥
const sharedSecret = 'shared-secret-123';
const dcmA = new DataChannelManager('peerA', sharedSecret);
const dcmB = new DataChannelManager('peerB', sharedSecret);
assert(dcmA.cryptoKey.equals(dcmB.cryptoKey), '相同sharedSecret派生相同密钥');
console.log(`  dcmA.cryptoKey: ${dcmA.cryptoKey.toString('hex').slice(0, 16)}...`);
console.log(`  dcmB.cryptoKey: ${dcmB.cryptoKey.toString('hex').slice(0, 16)}...`);

// 测试2: 不同sharedSecret派生不同密钥
const dcmC = new DataChannelManager('peerC', 'different-secret');
assert(!dcmA.cryptoKey.equals(dcmC.cryptoKey), '不同sharedSecret派生不同密钥');

// 测试3: 密钥可正常用于AES-256-GCM加密解密
const testMessage = 'Hello, P2P Encryption!';
const iv = crypto.randomBytes(16);
const cipher = crypto.createCipheriv('aes-256-gcm', dcmA.cryptoKey, iv);
const encrypted = Buffer.concat([cipher.update(testMessage, 'utf8'), cipher.final()]);
const authTag = cipher.getAuthTag();

const decipher = crypto.createDecipheriv('aes-256-gcm', dcmB.cryptoKey, iv);
decipher.setAuthTag(authTag);
const decrypted = Buffer.concat([decipher.update(encrypted), decipher.final()]);
assert(decrypted.toString('utf8') === testMessage, 'AES-256-GCM加密解密成功');
console.log(`  原文: ${testMessage}`);
console.log(`  解密: ${decrypted.toString('utf8')}`);

// 测试4: 无sharedSecret时抛出错误
try {
  new DataChannelManager('peerD', null);
  assert(false, '无sharedSecret时应抛出错误');
} catch (e) {
  assert(e.message.includes('sharedSecret is required'), '无sharedSecret时抛出正确错误');
  console.log(`  错误信息: ${e.message}`);
}

// 测试5: 密钥长度为32字节(256位)
assert(dcmA.cryptoKey.length === 32, '密钥长度为32字节(256位)');

console.log(`\n=== 测试结果: ${passed} 通过, ${failed} 失败 ===`);
process.exit(failed > 0 ? 1 : 0);
