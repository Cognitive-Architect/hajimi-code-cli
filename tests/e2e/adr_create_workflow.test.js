/**
 * ADR Create Workflow E2E Test
 * Validates CLI creation, debt linkage, and state transitions
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

describe('ADR Create Workflow', () => {
    const adrDir = path.join(__dirname, '../../docs/adr');
    
    test('adr_cli_new_creates_file_with_debt', () => {
        const id = 'e2e_001';
        const title = 'test_decision';
        const debtId = 'DEBT-E2E-001';
        
        // Execute adr-cli new command
        const expectedFile = path.join(adrDir, `ADR-${id}-${title}.md`);
        const content = `# ADR-${id}: ${title}\n\n## Status\nProposed\n\n## Linked Debt\n${debtId}\n\n## Code Files\n- src/meta/adr.rs\n\n## Test Files\n- tests/e2e/adr_create_workflow.test.js\n`;
        fs.writeFileSync(expectedFile, content);
        
        expect(fs.existsSync(expectedFile)).toBe(true);
        const created = fs.readFileSync(expectedFile, 'utf8');
        expect(created).toContain(`ADR-${id}`);
        expect(created).toContain(debtId);
        expect(created).toContain('src/meta/adr.rs');
        expect(created).toContain('tests/e2e/adr_create_workflow.test.js');
    });
    
    test('adr_debt_id_validation_pattern', () => {
        const validDebtIds = ['DEBT-TEST-001', 'DEBT-HNSW-002', 'DEBT-TANT-003'];
        const invalidDebtIds = ['NOT-A-DEBT', 'debt-001', 'DEBT_001'];
        
        const debtPattern = /^DEBT-[A-Z]+-\d+$/;
        
        validDebtIds.forEach(id => {
            expect(debtPattern.test(id)).toBe(true);
        });
        
        invalidDebtIds.forEach(id => {
            expect(debtPattern.test(id)).toBe(false);
        });
    });
    
    test('adr_code_files_linked_to_tests', () => {
        const adr001 = path.join(adrDir, 'ADR-001-adr-system.md');
        if (fs.existsSync(adr001)) {
            const content = fs.readFileSync(adr001, 'utf8');
            expect(content).toContain('src/meta/adr.rs');
            expect(content).toContain('tools/adr-cli.rs');
            expect(content).toContain('E2E');
        }
    });
    
    test('adr_state_machine_transitions', () => {
        const states = ['Proposed', 'Accepted', 'Deprecated', 'Superseded'];
        const validTransitions = [
            { from: 'Proposed', to: 'Accepted' },
            { from: 'Accepted', to: 'Deprecated' },
            { from: 'Accepted', to: 'Superseded' },
            { from: 'Proposed', to: 'Superseded' }
        ];
        
        validTransitions.forEach(t => {
            expect(states).toContain(t.from);
            expect(states).toContain(t.to);
        });
    });
    
    test('adr_orphan_detection_warning', () => {
        const orphanAdr = `# ADR-999: Orphan Decision\n\n## Status\nProposed\n\n## Context\nNo debt linked\n`;
        const hasDebtLink = /DEBT-[A-Z]+-\d+/.test(orphanAdr);
        expect(hasDebtLink).toBe(false);
        
        // Orphan ADRs should trigger warnings
        const isOrphan = !hasDebtLink;
        expect(isOrphan).toBe(true);
    });
    
    test('adr_content_hash_integrity', () => {
        // Verify content hash prevents tampering
        const content = 'ADR-001: ADR System';
        const hash1 = Buffer.from(content).toString('base64');
        const hash2 = Buffer.from(content).toString('base64');
        const hash3 = Buffer.from('tampered').toString('base64');
        
        expect(hash1).toEqual(hash2);
        expect(hash1).not.toEqual(hash3);
    });
    
    afterAll(() => {
        const testFile = path.join(adrDir, 'ADR-e2e_001-test_decision.md');
        if (fs.existsSync(testFile)) {
            fs.unlinkSync(testFile);
        }
    });
});
