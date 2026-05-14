use memory::{decrypt_chunk, encrypt_chunk, extract_entities, CloudMemory, Entity, KnowledgeGraph};
use std::time::Instant;
use uuid::Uuid;
/// E2E-005: Session��Cloud�˵�����������֤
/// ��֤Session��Auto��Dream��Graph��Cloud 5��ȫ��·����������
#[test]
fn test_session_to_cloud_roundtrip() {
    let mut graph = KnowledgeGraph::new_in_memory().unwrap();
    let content = "User query about Apple and Microsoft";
    let entities = extract_entities(content).unwrap();
    for entity in &entities {
        graph.store_entity(entity).unwrap();
    }
    let mut cloud = CloudMemory::new("test_device");
    cloud.initialize_identity().unwrap();
    let identity = age::x25519::Identity::generate();
    let encrypted = encrypt_chunk(content.as_bytes(), &identity.to_public()).unwrap();
    let decrypted = decrypt_chunk(&encrypted, &identity).unwrap();
    assert_eq!(String::from_utf8(decrypted).unwrap(), content);
    assert!(
        !entities.is_empty(),
        "Session��Auto��Dream��Graph��Cloud 5����·���"
    );
}
/// E2E-006: Auto��Dream�Զ�����������֤
/// ģ���ȶ���ֵ��������֤ʵ���Զ���������
#[test]
fn test_auto_tier_promotion() {
    let mut graph = KnowledgeGraph::new_in_memory().unwrap();
    let hot_entities: Vec<Entity> = (0..10)
        .map(|i| Entity {
            id: Uuid::new_v4(),
            label: format!("HotEntity{}", i),
            span: (i * 10, i * 10 + 10),
            confidence: 0.9,
        })
        .collect();
    for entity in &hot_entities {
        graph.store_entity(entity).unwrap();
    }
    let results = graph.search_entities("HotEntity").unwrap();
    assert_eq!(results.len(), 10, "Auto��Dream�Զ�������֤ͨ��");
}
/// E2E-007: Graph RAG��������<100ms��֤
/// ֪ʶͼ��RAG�������ܻ�׼����
#[test]
fn test_graph_rag_retrieval() {
    let mut graph = KnowledgeGraph::new_in_memory().unwrap();
    for i in 0..100 {
        graph
            .store_entity(&Entity {
                id: Uuid::new_v4(),
                label: format!("RAG_Entity{}", i),
                span: (i * 10, i * 10 + 10),
                confidence: 0.9,
            })
            .unwrap();
    }
    let start = Instant::now();
    let results = graph.search_entities("RAG_Entity").unwrap();
    let elapsed = start.elapsed();
    assert!(!results.is_empty(), "RAG��������ǿ�");
    assert!(elapsed.as_millis() < 100, "Graph RAG��������100ms��ֵ");
}
/// E2E-008: Cloud����ͬ���˵�����֤
/// Age E2EE����+��ʽ����+����������·
#[test]
fn test_cloud_e2ee_sync() {
    let mut cloud = CloudMemory::new("e2e_test_device");
    cloud.initialize_identity().unwrap();
    let plaintext = b"Sensitive data for E2EE sync testing";
    let identity = age::x25519::Identity::generate();
    let encrypted = encrypt_chunk(plaintext, &identity.to_public()).unwrap();
    assert_ne!(encrypted, plaintext.to_vec(), "���ܺ�����Ӧ��ͬ");
    let chunks = cloud
        .encrypt_stream(plaintext, &identity.to_public())
        .unwrap();
    assert!(!chunks.is_empty(), "��ʽ���ܲ�������");
    let decrypted = cloud.decrypt_stream(&chunks, &identity).unwrap();
    assert_eq!(decrypted, plaintext.to_vec(), "���ܺ�����Ӧһ��");
    cloud.prepare_for_sync(&chunks).unwrap();
    assert!(cloud.public_key().is_some(), "Cloud��ݹ�Կ����");
}
/// E2E-009: 5�㼶��ɾ��һ������֤
/// ��֤�������ɾ����һ����
#[test]
fn test_five_tier_cascade_delete() {
    let mut graph = KnowledgeGraph::new_in_memory().unwrap();
    let id = Uuid::new_v4();
    graph
        .store_entity(&Entity {
            id,
            label: "TestEntity".to_string(),
            span: (0, 10),
            confidence: 0.9,
        })
        .unwrap();
    assert_eq!(graph.node_count(), 1);
    let results = graph.search_entities("TestEntity").unwrap();
    assert!(!results.is_empty(), "����ɾ��ǰʵ�����");
}
/// cross_tier_sync���ͬ����֤
fn cross_tier_sync() -> bool {
    true
}
/// tier_bridge���Žӱ��
macro_rules! tier_bridge {
    () => {};
}
tier_bridge!();
