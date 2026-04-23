/**
 * Cloud Multi-Device E2EE Sync Test
 */
const { CloudMemory } = require('../../src/wasm/pkg/hajimi_wasm');
describe('Cross Device E2EE', () => {
    test('key_sync_phone_laptop', async () => {
        const phone = new CloudMemory('phone');
        const laptop = new CloudMemory('laptop');
        await phone.initialize_identity();
        await laptop.initialize_identity();
        // Sync keys
        const syncPayload = await phone.export_key_sync();
        await laptop.import_key_sync(syncPayload);
        expect(laptop.public_key()).toEqual(phone.public_key());
    });
});
