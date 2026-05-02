use codex_twist::memory::token_tracker::{TokenUsageTracker,SessionStats};
#[tokio::test]
async fn test_token_tracker_8round_multi_provider(){
    let t=TokenUsageTracker::new();
    let p=["openai","anthropic","ollama"];
    for i in 0..8{t.record_usage("s1",p[i%3],100+i as u64*10,50+i as u64*5).await;}
    let s=t.get_token_stats("s1").await;
    assert_eq!(s.request_count,8);
    assert_eq!(s.prompt_tokens,100+110+120+130+140+150+160+170);
    let g=t.get_global_stats().await;
    assert_eq!(g.total.request_count,8);
    assert!(g.by_provider.contains_key("openai")&&g.by_provider.contains_key("anthropic")&&g.by_provider.contains_key("ollama"));
}
#[tokio::test]
async fn test_token_tracker_session_accumulation(){
    let t=TokenUsageTracker::new();
    t.record_usage("s1","openai",100,50).await;
    t.record_usage("s1","openai",200,100).await;
    let s=t.get_token_stats("s1").await;
    assert_eq!(s.prompt_tokens,300);
    assert_eq!(s.request_count,2);
}
#[tokio::test]
async fn test_token_tracker_global_by_provider(){
    let t=TokenUsageTracker::new();
    t.record_usage("s1","openai",100,50).await;
    t.record_usage("s2","anthropic",200,100).await;
    let g=t.get_global_stats().await;
    assert_eq!(g.by_provider.get("openai").unwrap().request_count,1);
    assert_eq!(g.by_provider.get("anthropic").unwrap().request_count,1);
}
#[tokio::test]
async fn test_token_tracker_global_by_day(){
    let t=TokenUsageTracker::new();
    t.record_usage("s1","openai",100,50).await;
    t.record_usage("s2","openai",200,100).await;
    let g=t.get_global_stats().await;
    let today=chrono::Utc::now().format("%Y-%m-%d").to_string();
    assert_eq!(g.by_day.get(&today).unwrap().request_count,2);
}
#[tokio::test]
async fn test_token_tracker_concurrent_usage(){
    let t=TokenUsageTracker::new();
    let f1=t.record_usage("s1","openai",100,50);
    let f2=t.record_usage("s1","openai",200,100);
    tokio::join!(f1,f2);
    let s=t.get_token_stats("s1").await;
    assert_eq!(s.request_count,2);
    assert_eq!(s.prompt_tokens,300);
}
#[tokio::test]
async fn test_token_tracker_saturating_math(){
    let t=TokenUsageTracker::new();
    t.record_usage("s1","openai",u64::MAX,u64::MAX).await;
    t.record_usage("s1","openai",1,1).await;
    let s=t.get_token_stats("s1").await;
    assert_eq!(s.prompt_tokens,u64::MAX);
}
#[tokio::test]
async fn test_token_tracker_empty_session(){
    assert_eq!(TokenUsageTracker::new().get_token_stats("x").await,SessionStats::default());
}
#[tokio::test]
async fn test_token_tracker_precision_zero_error(){
    let t=TokenUsageTracker::new();
    let ep=1234u64;let ec=567u64;
    t.record_usage("s1","openai",ep,ec).await;
    let s=t.get_token_stats("s1").await;
    let err=if ep>0{((s.prompt_tokens as f64-ep as f64)/ep as f64).abs()*100.0}else{0.0};
    assert!(err<5.0,"error {:.2}% exceeds 5%",err);
    assert_eq!(s.prompt_tokens,ep);assert_eq!(s.completion_tokens,ec);
}
#[tokio::test]
async fn test_usage_stats_cumulative_consistency(){
    let t=TokenUsageTracker::new();
    t.record_usage("s1","openai",100,50).await;
    t.record_usage("s2","anthropic",200,100).await;
    t.record_usage("s3","ollama",50,25).await;
    let g=t.get_global_stats().await;
    let psum=g.by_provider.values().cloned().fold(SessionStats::default(),|mut a,s|{
        a.prompt_tokens+=s.prompt_tokens;a.completion_tokens+=s.completion_tokens;
        a.total_tokens+=s.total_tokens;a.request_count+=s.request_count;a
    });
    assert_eq!(psum,g.total);
}
#[tokio::test]
async fn test_token_tracker_provider_isolation(){
    let t=TokenUsageTracker::new();
    t.record_usage("s1","openai",100,50).await;
    t.record_usage("s1","anthropic",200,100).await;
    let g=t.get_global_stats().await;
    assert_eq!(g.by_provider.get("openai").unwrap().prompt_tokens,100);
    assert_eq!(g.by_provider.get("anthropic").unwrap().prompt_tokens,200);
}
#[tokio::test]
async fn test_usage_record_large_values(){
    let t=TokenUsageTracker::new();
    t.record_usage("s1","openai",1_000_000,500_000).await;
    t.record_usage("s1","openai",2_000_000,1_000_000).await;
    let s=t.get_token_stats("s1").await;
    assert_eq!(s.prompt_tokens,3_000_000);
    assert_eq!(s.total_tokens,4_500_000);
}
#[tokio::test]
async fn test_token_tracker_single_round(){
    let t=TokenUsageTracker::new();
    t.record_usage("s1","openai",42,21).await;
    let s=t.get_token_stats("s1").await;
    assert_eq!(s.prompt_tokens,42);
    assert_eq!(s.completion_tokens,21);
    assert_eq!(s.total_tokens,63);
}
