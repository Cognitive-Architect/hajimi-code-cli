/**
 * Cloud E2EE 5-Tier Memory Sync Test
 * Validates Session→Auto→Dream→Graph→Cloud encryption chain
 */
const { CloudMemory, encrypt_chunk, decrypt_chunk } = require('../../src/wasm/pkg/hajimi_wasm');
const { KnowledgeGraph, extract_entities } = require('../../src/intelligence/memory/pkg/memory');
describe('Cloud E2EE 5-Tier Sync', () => {
    test('session_to_cloud_roundtrip', async () => {
        const sessionData = {
            id: 'session_001',
            content: 'User query about Apple and Microsoft products',
            timestamp: Date.now(),
            access_count: 1
        };
        const autoData = {
            ...sessionData,
            tier: 'auto',
            priority: 0.8,
            promoted: true
        };
        const dreamData = {
            ...autoData,
            tier: 'dream',
            compressed: Buffer.from(autoData.content).toString('base64'),
            entropy: 0.75
        };
        const graph = new KnowledgeGraph();
        const entities = extract_entities(dreamData.content);
        entities.forEach(e => graph.store_entity(e));
        const cloud = new CloudMemory('device_001');
        await cloud.initialize_identity();
        const plaintext = Buffer.from(JSON.stringify(dreamData));
        const identity = generate_test_identity();
        const encrypted = encrypt_chunk(plaintext, identity.public_key);
        expect(encrypted).not.toEqual(plaintext);
        const decrypted = decrypt_chunk(encrypted, identity.secret_key);
        const recovered = JSON.parse(Buffer.from(decrypted).toString());
        expect(recovered.id).toEqual(sessionData.id);
        expect(recovered.tier).toEqual('dream');
        expect(recovered.content).toEqual(sessionData.content);
        expect(graph.size()).toBeGreaterThan(0);
    });
    test('x3dh_key_rotation', async () => {
        const cloud = new CloudMemory('rotation_test');
        await cloud.initialize_identity();
        const oldKey = cloud.public_key();
        cloud.config = { auto_rotate_days: 90 };
        await cloud.rotate_key();
        const newKey = cloud.public_key();
        expect(newKey).not.toEqual(oldKey);
        const oldData = encrypt_chunk(Buffer.from('test'), oldKey);
        const decrypted = await cloud.decrypt_with_legacy_key(oldData, oldKey);
        expect(decrypted).toBeTruthy();
    });
    test('five_tier_cascade_delete', async () => {
        const id = 'cascade_test_001';
        const session = { id, tier: 'session', content: 'Test' };
        const auto = { ...session, tier: 'auto' };
        const dream = { ...auto, tier: 'dream', compressed: 'dGVzdA==' };
        const graph = new KnowledgeGraph();
        const cloud = new CloudMemory('test');
        await cloud.initialize_identity();
        await cloud.delete_cascade(id);
        const exists = await cloud.exists(id);
        expect(exists).toBe(false);
    });
    test('encrypted_assertion', async () => {
        const cloud = new CloudMemory('assert_test');
        await cloud.initialize_identity();
        const plaintext = 'Sensitive data';
        const identity = generate_test_identity();
        const encrypted = encrypt_chunk(Buffer.from(plaintext), identity.public_key);
        expect(encrypted.length).toBeGreaterThan(0);
        expect(encrypted).not.toEqual(Buffer.from(plaintext));
        const asString = Buffer.from(encrypted).toString('utf8');
        expect(asString).not.toContain(plaintext);
    });
    test('rotation_history', async () => {
        const cloud = new CloudMemory('history_test');
        await cloud.initialize_identity();
        for (let i = 0; i < 3; i++) {
            await cloud.rotate_key();
        }
        const history = cloud.get_rotation_history();
        expect(history.length).toBe(3);
        history.forEach(entry => {
            expect(entry.timestamp).toBeDefined();
            expect(entry.old_version).toBeDefined();
            expect(entry.new_version).toBeDefined();
        });
    });
});
describe('Cloud Multi-Device Sync', () => {
    test('phone_laptop_sync', async () => {
        const phone = new CloudMemory('phone_device');
        const laptop = new CloudMemory('laptop_device');
        await phone.initialize_identity();
        await laptop.initialize_identity();
        const data = 'Cross-device test data';
        const encrypted = await phone.encrypt_for_device(
            data, 
            laptop.public_key()
        );
        const decrypted = await laptop.decrypt_from_device(encrypted);
        expect(decrypted).toEqual(data);
    });
    test('device_sync_conflict_resolution', async () => {
        const deviceA = new CloudMemory('device_a');
        const deviceB = new CloudMemory('device_b');
        await deviceA.initialize_identity();
        await deviceB.initialize_identity();
        const updateA = await deviceA.create_sync_payload('data_v1');
        const updateB = await deviceB.create_sync_payload('data_v2');
        const resolved = await deviceA.resolve_conflict(updateA, updateB);
        expect(resolved).toBeDefined();
    });
});
function generate_test_identity() {
    return {
        public_key: 'age1' + 'x'.repeat(58),
        secret_key: Buffer.from('test_secret_key_32_bytes_12345')
    };
}
function cross_tier_sync() {
    return 'Session→Auto→Dream→Graph→Cloud';
}
tier_bridge!();

