use std::time::Instant;
use tantivy::tokenizer::TextAnalyzer;

#[derive(Clone)]
struct JiebaTokenizer {
    jieba: jieba_rs::Jieba,
}

impl JiebaTokenizer {
    fn new() -> Self {
        Self {
            jieba: jieba_rs::Jieba::new(),
        }
    }
}

struct JiebaTokenStream {
    tokens: Vec<String>,
    current: usize,
    token: tantivy::tokenizer::Token,
}

impl tantivy::tokenizer::TokenStream for JiebaTokenStream {
    fn advance(&mut self) -> bool {
        if self.current < self.tokens.len() {
            let text = &self.tokens[self.current];
            self.token.text = text.clone();
            self.current += 1;
            true
        } else {
            false
        }
    }
    fn token(&self) -> &tantivy::tokenizer::Token {
        &self.token
    }
    fn token_mut(&mut self) -> &mut tantivy::tokenizer::Token {
        &mut self.token
    }
}

impl tantivy::tokenizer::Tokenizer for JiebaTokenizer {
    type TokenStream<'a> = JiebaTokenStream;
    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        let tokens = self
            .jieba
            .cut(text, true)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        JiebaTokenStream {
            tokens,
            current: 0,
            token: tantivy::tokenizer::Token::default(),
        }
    }
}

fn main() {
    let jieba = JiebaTokenizer::new();
    let analyzer = TextAnalyzer::builder(jieba).build();

    let t0 = Instant::now();
    let _c = analyzer.clone();
    println!("clone took {:?}", t0.elapsed());

    let t0 = Instant::now();
    let mut a = analyzer.clone();
    let _ = a.token_stream("latency");
    println!("token_stream took {:?}", t0.elapsed());

    let t0 = Instant::now();
    let j = jieba_rs::Jieba::new();
    let _ = j.cut("latency", true);
    println!("jieba cut took {:?}", t0.elapsed());
}
