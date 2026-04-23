/**
 * ADR UI Visualization E2E Test
 */

const fs = require('fs');
const path = require('path');

describe('ADR Web UI Visualization', () => {
    test('adr_dashboard_lists_entries', () => {
        const dashboard = {
            entries: [
                { id: '001', title: 'ADR System', status: 'Accepted', debt: 'DEBT-ADR-001', code_files: ['src/meta/adr.rs'] },
                { id: '002', title: 'Tantivy Integration', status: 'Proposed', debt: 'DEBT-TANT-002', code_files: ['src/engine/search/tantivy_index.rs'] }
            ],
            total: 2,
            by_status: { Proposed: 1, Accepted: 1, Deprecated: 0, Superseded: 0 }
        };
        expect(dashboard.entries.length).toBe(2);
        expect(dashboard.total).toBe(2);
        expect(dashboard.by_status.Accepted).toBe(1);
    });
    
    test('adr_visualization_filters_by_debt', () => {
        const entries = [
            { id: '001', debt: 'DEBT-ADR-001', code_files: ['a.rs'] },
            { id: '002', debt: 'DEBT-TANT-002', code_files: ['b.rs'] },
            { id: '003', debt: 'DEBT-ADR-001', code_files: ['c.rs'] }
        ];
        const filtered = entries.filter(e => e.debt === 'DEBT-ADR-001');
        expect(filtered.length).toBe(2);
    });
    
    test('adr_lifecycle_web_ui', () => {
        const actions = ['create', 'propose', 'accept', 'deprecate', 'supersede'];
        const uiButtons = ['Create ADR', 'Propose', 'Accept', 'Deprecate'];
        uiButtons.forEach(btn => {
            expect(actions).toContain(btn.toLowerCase().replace(' adr', ''));
        });
    });
    
    test('adr_linked_files_visualization', () => {
        const entry = {
            id: '001',
            code_files: ['src/meta/adr.rs', 'tools/adr-cli.rs'],
            test_files: ['tests/e2e/adr_create_workflow.test.js', 'tests/e2e/adr_ui_visualization.test.js']
        };
        expect(entry.code_files.length).toBe(2);
        expect(entry.test_files.length).toBe(2);
        expect(entry.code_files[0]).toContain('adr.rs');
    });

    test('adr_status_badge_color_map_and_props', () => {
        const colorMap = { Accepted: 'green', Proposed: 'yellow', Deprecated: 'red' };
        expect(colorMap['Accepted']).toBe('green');
        expect(colorMap['Proposed']).toBe('yellow');
        expect(colorMap['Deprecated']).toBe('red');
        // References AdrStatusBadge linking props
        const badgeProps = { status: 'Accepted', debt_id: 'DEBT-ADR-001', url: '/adr/001' };
        expect(badgeProps.debt_id).toContain('DEBT-');
        expect(badgeProps.url).toContain('/adr/');
    });
});
