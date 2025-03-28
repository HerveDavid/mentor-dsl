use rowan::{GreenNode, GreenNodeBuilder, Language};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MathLanguage {}

impl Language for MathLanguage {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::Root as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

pub type SyntaxNode = rowan::SyntaxNode<MathLanguage>;
type SyntaxToken = rowan::SyntaxToken<MathLanguage>;
type SyntaxElement = rowan::SyntaxElement<MathLanguage>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    // Tokens
    Number,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Whitespace,
    Error,

    // Expressions
    BinaryExpr,
    ParenExpr,
    LiteralExpr,

    // Root
    Root,
}

pub struct AstParser {
    builder: GreenNodeBuilder<'static>,
}

impl AstParser {
    pub fn new() -> Self {
        Self {
            builder: GreenNodeBuilder::new(),
        }
    }

    // Helper to start a new node
    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(MathLanguage::kind_to_raw(kind));
    }

    // Helper to finish a node
    fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    // Helper to add a token
    fn token(&mut self, kind: SyntaxKind, text: &str) {
        self.builder.token(MathLanguage::kind_to_raw(kind), text);
    }

    // Parse a number literal
    pub fn parse_number(&mut self, text: &str) {
        self.start_node(SyntaxKind::LiteralExpr);
        self.token(SyntaxKind::Number, text);
        self.finish_node();
    }

    // Parse a binary expression
    pub fn parse_binary_expr(&mut self, left: &str, op: &str, right: &str) {
        self.start_node(SyntaxKind::BinaryExpr);

        // Parse left
        match op {
            "+" | "-" => {
                // For add/sub, left can be any expression
                if left.contains('+') || left.contains('-') {
                    self.parse_expr(left);
                } else if left.contains('*') || left.contains('/') {
                    self.parse_expr(left);
                } else if left.contains('(') {
                    self.parse_paren_expr(left);
                } else {
                    self.parse_number(left);
                }
            }
            "*" | "/" => {
                // For mul/div, left should be a term
                if left.contains('*') || left.contains('/') {
                    self.parse_expr(left);
                } else if left.contains('(') {
                    self.parse_paren_expr(left);
                } else {
                    self.parse_number(left);
                }
            }
            _ => unreachable!(),
        }

        // Add operator token
        match op {
            "+" => self.token(SyntaxKind::Plus, op),
            "-" => self.token(SyntaxKind::Minus, op),
            "*" => self.token(SyntaxKind::Star, op),
            "/" => self.token(SyntaxKind::Slash, op),
            _ => unreachable!(),
        }

        // Parse right
        match op {
            "+" | "-" => {
                // For add/sub, right should be a term
                if right.contains('*') || right.contains('/') {
                    self.parse_expr(right);
                } else if right.contains('(') {
                    self.parse_paren_expr(right);
                } else {
                    self.parse_number(right);
                }
            }
            "*" | "/" => {
                // For mul/div, right should be a factor
                if right.contains('(') {
                    self.parse_paren_expr(right);
                } else {
                    self.parse_number(right);
                }
            }
            _ => unreachable!(),
        }

        self.finish_node();
    }

    // Parse a parenthesized expression
    pub fn parse_paren_expr(&mut self, text: &str) {
        self.start_node(SyntaxKind::ParenExpr);

        // Strip the outer parentheses
        let inner = &text[1..text.len() - 1].trim();

        self.token(SyntaxKind::LParen, "(");
        self.parse_expr(inner);
        self.token(SyntaxKind::RParen, ")");

        self.finish_node();
    }

    // Parse any expression (dispatcher)
    pub fn parse_expr(&mut self, text: &str) {
        let text = text.trim();

        // Try to find binary operators at the top level
        if let Some(idx) = find_top_level_operator(text, &['+', '-']) {
            let (left, right) = text.split_at(idx);
            let op = &right[0..1];
            let right = &right[1..];
            self.parse_binary_expr(left, op, right);
        } else if let Some(idx) = find_top_level_operator(text, &['*', '/']) {
            let (left, right) = text.split_at(idx);
            let op = &right[0..1];
            let right = &right[1..];
            self.parse_binary_expr(left, op, right);
        } else if text.starts_with('(') && text.ends_with(')') {
            self.parse_paren_expr(text);
        } else {
            // It's a number
            self.parse_number(text);
        }
    }

    // Parse a complete expression and return the syntax tree
    pub fn parse(&mut self, input: &str) -> SyntaxNode {
        self.start_node(SyntaxKind::Root);
        self.parse_expr(input);
        self.finish_node();

        // Avoid the move out of self.builder
        let builder = std::mem::replace(&mut self.builder, GreenNodeBuilder::new());
        let green = builder.finish();
        SyntaxNode::new_root(green)
    }
}

// Helper function to find top-level operators
fn find_top_level_operator(text: &str, ops: &[char]) -> Option<usize> {
    let chars: Vec<_> = text.chars().collect();
    let mut paren_level = 0;

    // Scan from right to left for + and - (precedence)
    for i in (0..chars.len()).rev() {
        let c = chars[i];

        if c == ')' {
            paren_level += 1;
        } else if c == '(' {
            paren_level -= 1;
        } else if paren_level == 0 && ops.contains(&c) {
            return Some(i);
        }
    }

    None
}

// Evaluate an AST node
pub fn eval(node: &SyntaxNode) -> Option<i32> {
    match node.kind() {
        SyntaxKind::Root => {
            // Evaluate the first child of the root
            for child in node.children() {
                return eval(&child);
            }
            None
        }
        SyntaxKind::BinaryExpr => {
            let mut children = node.children();

            let left = children.next()?;
            let op = node
                .children_with_tokens()
                .filter_map(|element| element.into_token())
                .find(|token| {
                    matches!(
                        token.kind(),
                        SyntaxKind::Plus | SyntaxKind::Minus | SyntaxKind::Star | SyntaxKind::Slash
                    )
                })?;
            let right = children.next()?;

            let left_val = eval(&left)?;
            let right_val = eval(&right)?;

            match op.kind() {
                SyntaxKind::Plus => Some(left_val + right_val),
                SyntaxKind::Minus => Some(left_val - right_val),
                SyntaxKind::Star => Some(left_val * right_val),
                SyntaxKind::Slash => {
                    if right_val == 0 {
                        None // Division by zero
                    } else {
                        Some(left_val / right_val)
                    }
                }
                _ => None,
            }
        }
        SyntaxKind::ParenExpr => {
            // Find the expression inside the parentheses
            for child in node.children() {
                if child.kind() != SyntaxKind::LiteralExpr
                    && child.kind() != SyntaxKind::BinaryExpr
                    && child.kind() != SyntaxKind::ParenExpr
                {
                    continue;
                }
                return eval(&child);
            }
            None
        }
        SyntaxKind::LiteralExpr => {
            // Extract the number token
            let token = node
                .children_with_tokens()
                .filter_map(|element| element.into_token())
                .find(|token| token.kind() == SyntaxKind::Number)?;

            token.text().parse::<i32>().ok()
        }
        _ => None,
    }
}

// Pretty print an AST
pub fn print_ast(node: &SyntaxNode, indent: usize) {
    let kind = format!("{:?}", node.kind());
    println!("{:indent$}{}", "", kind, indent = indent);

    // Print tokens directly attached to this node
    for element in node.children_with_tokens() {
        match element {
            rowan::NodeOrToken::Token(token) => {
                let token_kind = format!("{:?}", token.kind());
                let token_text = token.text();
                println!(
                    "{:indent$}  TOKEN {:?}: {:?}",
                    "",
                    token_kind,
                    token_text,
                    indent = indent
                );
            }
            rowan::NodeOrToken::Node(child) => {
                print_ast(&child, indent + 2);
            }
        }
    }
}
