use std::{
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
    iter::IntoIterator,
    ops::Deref,
    str::FromStr,
};

use crate::diagnostic::Span;
use crate::path::{OwnedTargetPath, OwnedValuePath, PathPrefix};
#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use ordered_float::NotNan;

use super::{Error, template_string::TemplateString};

/// Express syntactic rather than textual equality
pub trait SyntaxEq {
    /// true if self is syntactically equal to other
    fn syntax_eq(&self, other: &Self) -> bool;
}

impl<T: SyntaxEq> SyntaxEq for Vec<Node<T>> {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .enumerate()
                .all(|(index, element)| element.syntax_eq(&other[index]))
    }
}

impl<T: SyntaxEq> SyntaxEq for Option<T> {
    fn syntax_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (None, None) => true,
            (Some(n1), Some(n2)) => n1.syntax_eq(n2),
            (_, _) => false,
        }
    }
}

impl<T: SyntaxEq> SyntaxEq for Box<T> {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.as_ref().syntax_eq(other.as_ref())
    }
}

// -----------------------------------------------------------------------------
// node
// -----------------------------------------------------------------------------

/// A wrapper type for a node, containing span details of that given node as it
/// relates to the source input from which the node was generated.
#[derive(Clone, Eq, Ord, PartialOrd)]
pub struct Node<T> {
    pub(crate) span: Span,
    pub(crate) node: T,
}

impl<T> Node<T> {
    pub fn map<R>(self, mut f: impl FnMut(T) -> R) -> Node<R> {
        let Node { span, node } = self;

        Node {
            span,
            node: f(node),
        }
    }

    pub fn map_option<R>(self, mut f: impl FnMut(T) -> Option<R>) -> Option<Node<R>> {
        let Node { span, node } = self;

        let node = f(node)?;

        Some(Node { span, node })
    }

    pub fn new(span: Span, node: T) -> Self {
        Self { span, node }
    }

    /// Get a copy of the [`Span`] of the node.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Get the starting byte of the node within the input source.
    pub fn start(&self) -> usize {
        self.span.start()
    }

    /// Get the ending byte of the node within the input source.
    pub fn end(&self) -> usize {
        self.span.end()
    }

    /// Get a reference to the inner node type `T`.
    pub fn inner(&self) -> &T {
        &self.node
    }

    // Consume the node, taking out the [`Span`] and inner node type `T`.
    pub fn take(self) -> (Span, T) {
        (self.span, self.node)
    }

    /// Consume the node, and get the inner node type `T`.
    pub fn into_inner(self) -> T {
        self.node
    }

    /// Consume the node and return a tuple consisting of the start, node type
    /// `T` and the end position.
    pub fn into_spanned(self) -> (usize, T, usize) {
        let Self { span, node } = self;

        (span.start(), node, span.end())
    }

    pub fn as_deref(&self) -> &T::Target
    where
        T: Deref,
    {
        self.as_ref()
    }
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> AsRef<T> for Node<T> {
    fn as_ref(&self) -> &T {
        &self.node
    }
}

impl<T: PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.span == other.span
    }
}

impl<T: Hash> Hash for Node<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state);
        self.span.hash(state);
    }
}

impl<T: SyntaxEq> SyntaxEq for Node<T> {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.node.syntax_eq(&other.node)
    }
}

// -----------------------------------------------------------------------------
// program
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Program(pub Vec<Node<RootExpr>>);

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for expr in &self.0 {
            writeln!(f, "{expr:?}")?;
        }

        Ok(())
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for expr in &self.0 {
            writeln!(f, "{expr}")?;
        }

        Ok(())
    }
}

impl Deref for Program {
    type Target = [Node<RootExpr>];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl IntoIterator for Program {
    type Item = Node<RootExpr>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl SyntaxEq for Program {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.0.syntax_eq(&other.0)
    }
}

// -----------------------------------------------------------------------------
// root expression
// -----------------------------------------------------------------------------

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq)]
pub enum RootExpr {
    Expr(Node<Expr>),

    /// A special expression that is returned if a given expression could not be
    /// parsed. This allows the parser to continue on to the next expression.
    Error(Error),
}

impl fmt::Debug for RootExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RootExpr::{Error, Expr};

        let value = match self {
            Expr(v) => format!("{v:?}"),
            Error(v) => format!("{v:?}"),
        };

        write!(f, "RootExpr({value})")
    }
}

impl fmt::Display for RootExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RootExpr::{Error, Expr};

        match self {
            Expr(v) => v.fmt(f),
            Error(v) => v.fmt(f),
        }
    }
}

impl SyntaxEq for RootExpr {
    fn syntax_eq(&self, other: &Self) -> bool {
        use RootExpr::{Error, Expr};

        match (self, other) {
            (Expr(e1), Expr(e2)) => e1.syntax_eq(e2),
            (Error(e1), Error(e2)) => e1 == e2,
            (_, _) => false,
        }
    }
}

// -----------------------------------------------------------------------------
// expression
// -----------------------------------------------------------------------------

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq)]
pub enum Expr {
    Literal(Node<Literal>),
    Container(Node<Container>),
    IfStatement(Node<IfStatement>),
    Op(Node<Op>),
    Assignment(Node<Assignment>),
    Query(Node<Query>),
    FunctionCall(Node<FunctionCall>),
    Variable(Node<Ident>),
    Unary(Node<Unary>),
    Abort(Node<Abort>),
    Return(Node<Return>),
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::{
            Abort, Assignment, Container, FunctionCall, IfStatement, Literal, Op, Query, Return,
            Unary, Variable,
        };

        let value = match self {
            Literal(v) => format!("{v:?}"),
            Container(v) => format!("{v:?}"),
            Op(v) => format!("{v:?}"),
            IfStatement(v) => format!("{v:?}"),
            Assignment(v) => format!("{v:?}"),
            Query(v) => format!("{v:?}"),
            FunctionCall(v) => format!("{v:?}"),
            Variable(v) => format!("{v:?}"),
            Unary(v) => format!("{v:?}"),
            Abort(v) => format!("{v:?}"),
            Return(v) => format!("{v:?}"),
        };

        write!(f, "Expr({value})")
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::{
            Abort, Assignment, Container, FunctionCall, IfStatement, Literal, Op, Query, Return,
            Unary, Variable,
        };

        match self {
            Literal(v) => v.fmt(f),
            Container(v) => v.fmt(f),
            Op(v) => v.fmt(f),
            IfStatement(v) => v.fmt(f),
            Assignment(v) => v.fmt(f),
            Query(v) => v.fmt(f),
            FunctionCall(v) => v.fmt(f),
            Variable(v) => v.fmt(f),
            Unary(v) => v.fmt(f),
            Abort(v) => v.fmt(f),
            Return(v) => v.fmt(f),
        }
    }
}

impl SyntaxEq for Expr {
    fn syntax_eq(&self, other: &Self) -> bool {
        use Expr::{
            Abort, Assignment, Container, FunctionCall, IfStatement, Literal, Op, Query, Return,
            Unary, Variable,
        };

        match (self, other) {
            (Literal(v), Literal(v2)) => v.syntax_eq(v2),
            (Container(v), Container(v2)) => v.syntax_eq(v2),
            (Op(v), Op(v2)) => v.syntax_eq(v2),
            (IfStatement(v), IfStatement(v2)) => v.syntax_eq(v2),
            (Assignment(v), Assignment(v2)) => v.syntax_eq(v2),
            (Query(v), Query(v2)) => v.syntax_eq(v2),
            (FunctionCall(v), FunctionCall(v2)) => v.syntax_eq(v2),
            (Variable(v), Variable(v2)) => v.syntax_eq(v2),
            (Unary(v), Unary(v2)) => v.syntax_eq(v2),
            (Abort(v), Abort(v2)) => v.syntax_eq(v2),
            (Return(v), Return(v2)) => v.syntax_eq(v2),
            (_, _) => false,
        }
    }
}

// -----------------------------------------------------------------------------
// ident
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ident(pub(crate) String);

impl Ident {
    pub fn new(ident: impl Into<String>) -> Self {
        Self(ident.into())
    }

    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ident({})", self.0)
    }
}

impl From<String> for Ident {
    fn from(ident: String) -> Self {
        Ident(ident)
    }
}

impl SyntaxEq for Ident {
    fn syntax_eq(&self, other: &Self) -> bool {
        self == other
    }
}

// -----------------------------------------------------------------------------
// literals
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq)]
pub enum Literal {
    String(TemplateString),
    RawString(String),
    Integer(i64),
    Float(NotNan<f64>),
    Boolean(bool),
    Regex(String),
    Timestamp(String),
    Null,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Literal::{Boolean, Float, Integer, Null, RawString, Regex, String, Timestamp};

        match self {
            String(v) => write!(f, r#""{v}""#),
            RawString(v) => write!(f, "s'{v}'"),
            Integer(v) => v.fmt(f),
            Float(v) => v.fmt(f),
            Boolean(v) => v.fmt(f),
            Regex(v) => write!(f, "r'{v}'"),
            Timestamp(v) => write!(f, "t'{v}'"),
            Null => f.write_str("null"),
        }
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Literal({self})")
    }
}

impl SyntaxEq for Literal {
    fn syntax_eq(&self, other: &Self) -> bool {
        use Literal::String;
        match (self, other) {
            (String(t1), String(t2)) => t1.to_string() == t2.to_string(),
            (o1, o2) => o1 == o2,
        }
    }
}

// -----------------------------------------------------------------------------
// container
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub enum Container {
    Group(Box<Node<Group>>),
    Block(Node<Block>),
    Array(Node<Array>),
    Object(Node<Object>),
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Container::{Array, Block, Group, Object};

        match self {
            Group(v) => v.fmt(f),
            Block(v) => v.fmt(f),
            Array(v) => v.fmt(f),
            Object(v) => v.fmt(f),
        }
    }
}

impl fmt::Debug for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Container::{Array, Block, Group, Object};

        let value = match self {
            Group(v) => format!("{v:?}"),
            Block(v) => format!("{v:?}"),
            Array(v) => format!("{v:?}"),
            Object(v) => format!("{v:?}"),
        };

        write!(f, "Container({value})")
    }
}

impl SyntaxEq for Container {
    fn syntax_eq(&self, other: &Self) -> bool {
        use Container::{Array, Block, Group, Object};

        match (self, other) {
            (Group(o1), Group(o2)) => o1.syntax_eq(o2),
            (Block(o1), Block(o2)) => o1.syntax_eq(o2),
            (Array(o1), Array(o2)) => o1.syntax_eq(o2),
            (Object(o1), Object(o2)) => o1.syntax_eq(o2),
            (_, _) => false,
        }
    }
}

// -----------------------------------------------------------------------------
// block
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Block(pub Vec<Node<Expr>>);

impl Block {
    #[must_use]
    pub fn into_inner(self) -> Vec<Node<Expr>> {
        self.0
    }
}

impl IntoIterator for Block {
    type Item = Node<Expr>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{\n")?;

        let mut iter = self.0.iter().peekable();
        while let Some(expr) = iter.next() {
            f.write_str("\t")?;
            expr.fmt(f)?;
            if iter.peek().is_some() {
                f.write_str("\n")?;
            }
        }

        f.write_str("\n}")
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Block(")?;

        let mut iter = self.0.iter().peekable();
        while let Some(expr) = iter.next() {
            expr.fmt(f)?;

            if iter.peek().is_some() {
                f.write_str("; ")?;
            }
        }

        f.write_str(")")
    }
}

impl SyntaxEq for Block {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.0.syntax_eq(&other.0)
    }
}

// -----------------------------------------------------------------------------
// group
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Group(pub Node<Expr>);

impl Group {
    #[must_use]
    pub fn into_inner(self) -> Node<Expr> {
        self.0
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.0)
    }
}

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Group({:?})", self.0)
    }
}

impl SyntaxEq for Group {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.0.syntax_eq(&other.0)
    }
}

// -----------------------------------------------------------------------------
// array
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Array(pub(crate) Vec<Node<Expr>>);

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exprs = self
            .0
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "[{exprs}]")
    }
}

impl fmt::Debug for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exprs = self
            .0
            .iter()
            .map(|e| format!("{e:?}"))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "Array([{exprs}])")
    }
}

impl IntoIterator for Array {
    type Item = Node<Expr>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Node<Expr>> for Array {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Node<Expr>>,
    {
        Self(iter.into_iter().collect())
    }
}

impl SyntaxEq for Array {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.0.syntax_eq(&other.0)
    }
}

// -----------------------------------------------------------------------------
// object
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Object(pub(crate) BTreeMap<Node<String>, Node<Expr>>);

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exprs = self
            .0
            .iter()
            .map(|(k, v)| format!(r#""{k}": {v}"#))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{{ {exprs} }}")
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exprs = self
            .0
            .iter()
            .map(|(k, v)| format!(r#""{k}": {v:?}"#))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{{ {exprs} }}")
    }
}

impl IntoIterator for Object {
    type Item = (Node<String>, Node<Expr>);
    type IntoIter = std::collections::btree_map::IntoIter<Node<String>, Node<Expr>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(Node<String>, Node<Expr>)> for Object {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Node<String>, Node<Expr>)>,
    {
        Self(iter.into_iter().collect())
    }
}

impl SyntaxEq for Object {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self.0.iter().all(|(self_key, self_value)| {
                other.0.iter().any(|(other_key, other_value)| {
                    other_key.inner() == self_key.inner() && self_value.syntax_eq(other_value)
                })
            })
    }
}

// -----------------------------------------------------------------------------
// if statement
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct IfStatement {
    pub predicate: Node<Predicate>,
    pub if_node: Node<Block>,
    pub else_node: Option<Node<Block>>,
}

impl fmt::Debug for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.else_node {
            Some(alt) => write!(f, "{:?} ? {:?} : {alt:?}", self.predicate, self.if_node),
            None => write!(f, "{:?} ? {:?}", self.predicate, self.if_node),
        }
    }
}

impl fmt::Display for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("if ")?;
        self.predicate.fmt(f)?;
        f.write_str(" ")?;
        self.if_node.fmt(f)?;

        if let Some(alt) = &self.else_node {
            f.write_str(" else")?;
            alt.fmt(f)?;
        }

        Ok(())
    }
}

impl SyntaxEq for IfStatement {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.predicate.syntax_eq(&other.predicate)
            && self.if_node.syntax_eq(&other.if_node)
            && self.else_node.syntax_eq(&other.else_node)
    }
}

#[derive(Clone, PartialEq)]
pub enum Predicate {
    One(Box<Node<Expr>>),
    Many(Vec<Node<Expr>>),
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Predicate::One(expr) => expr.fmt(f),
            Predicate::Many(exprs) => {
                f.write_str("(")?;

                let mut iter = exprs.iter().peekable();
                while let Some(expr) = iter.next() {
                    expr.fmt(f)?;

                    if iter.peek().is_some() {
                        f.write_str("; ")?;
                    }
                }

                f.write_str(")")
            }
        }
    }
}

impl fmt::Debug for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Predicate::One(expr) => write!(f, "Predicate({expr:?})"),
            Predicate::Many(exprs) => {
                f.write_str("Predicate(")?;

                let mut iter = exprs.iter().peekable();
                while let Some(expr) = iter.next() {
                    expr.fmt(f)?;

                    if iter.peek().is_some() {
                        f.write_str("; ")?;
                    }
                }

                f.write_str(")")
            }
        }
    }
}

impl SyntaxEq for Predicate {
    fn syntax_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Predicate::One(n1), Predicate::One(n2)) => n1.syntax_eq(n2),
            (Predicate::Many(n1), Predicate::Many(n2)) => n1.syntax_eq(n2),
            (_, _) => false,
        }
    }
}

// -----------------------------------------------------------------------------
// operation
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Op(pub Box<Node<Expr>>, pub Node<Opcode>, pub Box<Node<Expr>>);

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.0, self.1, self.2)
    }
}

impl fmt::Debug for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Op({:?} {} {:?})", self.0, self.1, self.2)
    }
}

impl SyntaxEq for Op {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.0.syntax_eq(&other.0) && self.1.syntax_eq(&other.1) && self.2.syntax_eq(&other.2)
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Opcode {
    Mul,
    Div,
    Add,
    Sub,
    Or,
    And,
    Err,
    Ne,
    Eq,
    Ge,
    Gt,
    Le,
    Lt,
    Merge,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Opcode {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        use Opcode::{Add, And, Div, Eq, Err, Ge, Gt, Le, Lt, Merge, Mul, Ne, Or, Sub};

        match self {
            Mul => "*",
            Div => "/",
            Add => "+",
            Sub => "-",
            Merge => "|",

            Or => "||",
            And => "&&",

            Err => "??",

            Ne => "!=",
            Eq => "==",

            Ge => ">=",
            Gt => ">",
            Le => "<=",
            Lt => "<",
        }
    }
}

impl FromStr for Opcode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        use Opcode::{Add, And, Div, Eq, Err, Ge, Gt, Le, Lt, Merge, Mul, Ne, Or, Sub};

        let op = match s {
            "*" => Mul,
            "/" => Div,
            "+" => Add,
            "-" => Sub,

            "||" => Or,
            "&&" => And,

            "??" => Err,

            "!=" => Ne,
            "==" => Eq,

            ">=" => Ge,
            ">" => Gt,
            "<=" => Le,
            "<" => Lt,
            "|" => Merge,

            _ => return std::result::Result::Err(()),
        };

        Ok(op)
    }
}

impl SyntaxEq for Opcode {
    fn syntax_eq(&self, other: &Self) -> bool {
        self == other
    }
}

// -----------------------------------------------------------------------------
// assignment
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Assignment {
    Single {
        target: Node<AssignmentTarget>,
        op: AssignmentOp,
        expr: Box<Node<Expr>>,
    },
    Infallible {
        ok: Node<AssignmentTarget>,
        err: Node<AssignmentTarget>,
        op: AssignmentOp,
        expr: Box<Node<Expr>>,
    },
    // TODO
    // Compound {
    //     target: Node<AssignmentTarget>,
    //     op: Opcode,
    //     expr: Box<Node<Expr>>,
    // }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[derive(Clone, PartialEq, Eq)]
pub enum AssignmentOp {
    Assign,
    Merge,
}

impl fmt::Display for AssignmentOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AssignmentOp::{Assign, Merge};

        match self {
            Assign => write!(f, "="),
            Merge => write!(f, "|="),
        }
    }
}

impl fmt::Debug for AssignmentOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AssignmentOp::{Assign, Merge};

        match self {
            Assign => write!(f, "AssignmentOp(=)"),
            Merge => write!(f, "AssignmentOp(|=)"),
        }
    }
}

impl SyntaxEq for AssignmentOp {
    fn syntax_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Assignment::{Infallible, Single};

        match self {
            Single { target, op, expr } => write!(f, "{target} {op} {expr}"),
            Infallible { ok, err, op, expr } => write!(f, "{ok}, {err} {op} {expr}"),
        }
    }
}

impl fmt::Debug for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Assignment::{Infallible, Single};

        match self {
            Single { target, op, expr } => write!(f, "{target:?} {op:?} {expr:?}"),
            Infallible { ok, err, op, expr } => {
                write!(f, "Ok({ok:?}), Err({err:?}) {op:?} {expr:?}")
            }
        }
    }
}

impl SyntaxEq for Assignment {
    fn syntax_eq(&self, other: &Self) -> bool {
        use Assignment::{Infallible, Single};

        match (self, other) {
            (
                Single {
                    target: t1,
                    op: o1,
                    expr: e1,
                },
                Single {
                    target: t2,
                    op: o2,
                    expr: e2,
                },
            ) => t1.syntax_eq(t2) && o1.syntax_eq(o2) && e1.syntax_eq(e2),
            (
                Infallible {
                    ok: ok1,
                    err: e1,
                    op: op1,
                    expr: ex1,
                },
                Infallible {
                    ok: ok2,
                    err: e2,
                    op: op2,
                    expr: ex2,
                },
            ) => ok1.syntax_eq(ok2) && e1.syntax_eq(e2) && op1.syntax_eq(op2) && ex1.syntax_eq(ex2),
            (_, _) => false,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum AssignmentTarget {
    Noop,
    Query(Query),
    Internal(Ident, Option<OwnedValuePath>),
    External(Option<OwnedTargetPath>),
}

impl AssignmentTarget {
    #[must_use]
    pub fn to_expr(&self, span: Span) -> Expr {
        match self {
            AssignmentTarget::Noop => Expr::Literal(Node::new(span, Literal::Null)),
            AssignmentTarget::Query(query) => Expr::Query(Node::new(span, query.clone())),
            AssignmentTarget::Internal(ident, Some(path)) => Expr::Query(Node::new(
                span,
                Query {
                    target: Node::new(span, QueryTarget::Internal(ident.clone())),
                    path: Node::new(span, path.clone()),
                },
            )),
            AssignmentTarget::Internal(ident, None) => {
                Expr::Variable(Node::new(span, ident.clone()))
            }
            AssignmentTarget::External(path) => Expr::Query(Node::new(
                span,
                Query {
                    target: {
                        let prefix = path.as_ref().map_or(PathPrefix::Event, |x| x.prefix);
                        Node::new(span, QueryTarget::External(prefix))
                    },
                    path: Node::new(
                        span,
                        path.clone().map_or(OwnedValuePath::root(), |x| x.path),
                    ),
                },
            )),
        }
    }
}

impl fmt::Display for AssignmentTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AssignmentTarget::{External, Internal, Noop, Query};

        match self {
            Noop => f.write_str("_"),
            Query(query) => query.fmt(f),
            Internal(ident, Some(path)) => write!(f, "{ident}{path}"),
            Internal(ident, _) => ident.fmt(f),
            External(Some(path)) => write!(f, "{path}"),
            External(_) => f.write_str("."),
        }
    }
}

impl fmt::Debug for AssignmentTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AssignmentTarget::{External, Internal, Noop, Query};

        match self {
            Noop => f.write_str("Noop"),
            Query(query) => query.fmt(f),
            Internal(ident, Some(path)) => write!(f, "Internal({ident}{path})"),
            Internal(ident, _) => write!(f, "Internal({ident})"),
            External(Some(path)) => write!(f, "External({path})"),
            External(_) => f.write_str("External(.)"),
        }
    }
}

impl SyntaxEq for AssignmentTarget {
    fn syntax_eq(&self, other: &Self) -> bool {
        use AssignmentTarget::{External, Internal, Noop, Query};

        match (self, other) {
            (Noop, Noop) => true,
            (Query(q1), Query(q2)) => {
                q1.target.syntax_eq(&q2.target) && q1.path.inner() == q2.path.inner()
            }
            (Internal(Ident(s1), p1), Internal(Ident(s2), p2)) => s1 == s2 && p1 == p2,
            (External(p1), External(p2)) => p1 == p2,
            (_, _) => false,
        }
    }
}

// -----------------------------------------------------------------------------
// query
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Query {
    pub target: Node<QueryTarget>,
    pub path: Node<OwnedValuePath>,
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.target, self.path)
    }
}

impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Query({:?}, {:?})", self.target, self.path)
    }
}

impl SyntaxEq for Query {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.target.syntax_eq(&other.target) && self.path.inner() == other.path.inner()
    }
}

#[derive(Clone, PartialEq)]
pub enum QueryTarget {
    Internal(Ident),
    External(PathPrefix),
    FunctionCall(FunctionCall),
    Container(Container),
}

impl fmt::Display for QueryTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QueryTarget::{Container, External, FunctionCall, Internal};

        match self {
            Internal(v) => v.fmt(f),
            External(prefix) => match prefix {
                PathPrefix::Event => write!(f, "."),
                PathPrefix::Metadata => write!(f, "&"),
            },
            FunctionCall(v) => v.fmt(f),
            Container(v) => v.fmt(f),
        }
    }
}

impl fmt::Debug for QueryTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QueryTarget::{Container, External, FunctionCall, Internal};

        match self {
            Internal(v) => write!(f, "Internal({v:?})"),
            External(prefix) => match prefix {
                PathPrefix::Event => f.write_str("External(Event)"),
                PathPrefix::Metadata => f.write_str("External(Metadata)"),
            },
            FunctionCall(v) => v.fmt(f),
            Container(v) => v.fmt(f),
        }
    }
}

impl SyntaxEq for QueryTarget {
    fn syntax_eq(&self, other: &Self) -> bool {
        use QueryTarget::{Container, External, FunctionCall, Internal};

        match (self, other) {
            (Internal(ident1), Internal(ident2)) => ident1 == ident2,
            (External(prefix1), External(prefix2)) => prefix1 == prefix2,
            (Container(c1), Container(c2)) => c1.syntax_eq(c2),
            (FunctionCall(f1), FunctionCall(f2)) => f1.syntax_eq(f2),
            (_, _) => false,
        }
    }
}

// -----------------------------------------------------------------------------
// function call
// -----------------------------------------------------------------------------

/// A function call expression.
///
/// It contains the identifier of the function, and any arguments passed into
/// the function call.
#[derive(Clone, PartialEq)]
pub struct FunctionCall {
    pub ident: Node<Ident>,
    pub abort_on_error: bool,
    pub arguments: Vec<Node<FunctionArgument>>,
    pub closure: Option<Node<FunctionClosure>>,
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ident.fmt(f)?;
        f.write_str("(")?;

        let mut iter = self.arguments.iter().peekable();
        while let Some(arg) = iter.next() {
            arg.fmt(f)?;

            if iter.peek().is_some() {
                f.write_str(", ")?;
            }
        }

        f.write_str(")")?;

        if let Some(closure) = &self.closure {
            f.write_str(" ")?;
            closure.fmt(f)?;
        }

        Ok(())
    }
}

impl fmt::Debug for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FunctionCall(")?;
        self.ident.fmt(f)?;

        f.write_str("(")?;

        let mut iter = self.arguments.iter().peekable();
        while let Some(arg) = iter.next() {
            arg.fmt(f)?;

            if iter.peek().is_some() {
                f.write_str(", ")?;
            }
        }

        f.write_str(")")?;

        if let Some(closure) = &self.closure {
            f.write_str(" ")?;
            closure.fmt(f)?;
        }

        f.write_str(")")
    }
}

impl SyntaxEq for FunctionCall {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.ident.syntax_eq(&other.ident)
            && self.abort_on_error == other.abort_on_error
            && self.arguments.syntax_eq(&other.arguments)
            && self.closure.syntax_eq(&other.closure)
    }
}

/// An argument passed to a function call.
///
/// The first value is an optional identifier provided for the argument, making
/// it a _keyword argument_ as opposed to a _positional argument_.
///
/// The second value is the expression provided as the argument.
#[derive(Clone, PartialEq)]
pub struct FunctionArgument {
    pub ident: Option<Node<Ident>>,
    pub expr: Node<Expr>,
}

impl fmt::Display for FunctionArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ident) = &self.ident {
            write!(f, "{ident}: ")?;
        }

        self.expr.fmt(f)
    }
}

impl fmt::Debug for FunctionArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ident) = &self.ident {
            write!(f, "Argument({ident:?}: {:?})", self.expr)
        } else {
            write!(f, "Argument({:?})", self.expr)
        }
    }
}

impl SyntaxEq for FunctionArgument {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.ident.syntax_eq(&other.ident) && self.expr.syntax_eq(&other.expr)
    }
}

/// A closure attached to a function.
#[derive(Clone, PartialEq)]
pub struct FunctionClosure {
    pub variables: Vec<Node<Ident>>,
    pub block: Node<Block>,
}

impl fmt::Display for FunctionClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("-> |")?;

        let mut iter = self.variables.iter().peekable();
        while let Some(var) = iter.next() {
            var.fmt(f)?;

            if iter.peek().is_some() {
                f.write_str(", ")?;
            }
        }

        f.write_str("| {\n")?;

        let mut iter = self.block.0.iter().peekable();
        while let Some(expr) = iter.next() {
            f.write_str("\t")?;
            expr.fmt(f)?;
            if iter.peek().is_some() {
                f.write_str("\n")?;
            }
        }

        f.write_str("\n}")
    }
}

impl fmt::Debug for FunctionClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Closure(...)")
    }
}

impl SyntaxEq for FunctionClosure {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.variables.syntax_eq(&other.variables) && self.block.syntax_eq(&other.block)
    }
}

// -----------------------------------------------------------------------------
// unary
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub enum Unary {
    Not(Node<Not>),
}

impl fmt::Display for Unary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Unary::Not;

        match self {
            Not(v) => v.fmt(f),
        }
    }
}

impl fmt::Debug for Unary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Unary::Not;

        let value = match self {
            Not(v) => format!("{v:?}"),
        };

        write!(f, "Unary({value})")
    }
}

impl SyntaxEq for Unary {
    fn syntax_eq(&self, other: &Self) -> bool {
        use Unary::Not;

        match (self, other) {
            (Not(node1), Not(node2)) => node1.syntax_eq(node2),
        }
    }
}

// -----------------------------------------------------------------------------
// not
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Not(pub(crate) Node<()>, pub(crate) Box<Node<Expr>>);

impl Not {
    #[must_use]
    pub fn take(self) -> (Node<()>, Box<Node<Expr>>) {
        (self.0, self.1)
    }

    #[must_use]
    pub fn new(span: Span, expr: Node<Expr>) -> Self {
        Self(Node::new(span, ()), Box::new(expr))
    }
}

impl fmt::Display for Not {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "!{}", self.1)
    }
}

impl fmt::Debug for Not {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Not({:?})", self.1)
    }
}

impl SyntaxEq for Not {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.1.syntax_eq(&other.1)
    }
}

// -----------------------------------------------------------------------------
// abort
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Abort {
    pub message: Option<Box<Node<Expr>>>,
}

impl fmt::Display for Abort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &self
                .message
                .as_ref()
                .map_or_else(|| "abort".to_owned(), |m| format!("abort: {m}")),
        )
    }
}

impl fmt::Debug for Abort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Abort({:?})", self.message)
    }
}

impl SyntaxEq for Abort {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.message.syntax_eq(&other.message)
    }
}

// -----------------------------------------------------------------------------
// return
// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
pub struct Return {
    pub expr: Box<Node<Expr>>,
}

impl fmt::Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "return {}", self.expr)
    }
}

impl fmt::Debug for Return {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Return({:?})", self.expr)
    }
}

impl SyntaxEq for Return {
    fn syntax_eq(&self, other: &Self) -> bool {
        self.expr.syntax_eq(&other.expr)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    use super::SyntaxEq;

    #[test]
    fn comments_and_whitespace_do_not_affect_syntactic_equality() {
        let (prog1, prog2) = (
            parse(
                r".x[1].y = sum(1, 2)
x = read(.items) -> |item| {
  item * 2
}

if x == null {
    abort
}
return x + 1
",
            )
            .unwrap(),
            parse(
                r"# adding together
.x[1].y =  sum(1, 2)

x = read(.items) -> |item| {
  item * 2 # doubling
}
if x == null {
    abort
}

return x + 1
",
            )
            .unwrap(),
        );
        assert_ne!(prog1, prog2);
        assert!(prog1.syntax_eq(&prog2))
    }

    #[test]
    fn abort_message_changes_syntactic_equality() {
        let (prog1, prog2) = (
            parse(r#"abort "bad apple""#).unwrap(),
            parse(r#"abort "no apple""#).unwrap(),
        );
        assert_ne!(prog1, prog2);
        assert!(!prog1.syntax_eq(&prog2));

        let (prog1, prog2) = (parse("abort").unwrap(), parse("abort \"\"").unwrap());
        assert_ne!(prog1, prog2);
        assert!(!prog1.syntax_eq(&prog2));
    }

    #[test]
    fn function_arg_changes_syntactic_equality() {
        let (prog1, prog2) = (
            parse(r#"run("foo", 2)"#).unwrap(),
            parse(r#"run("foo", 3)"#).unwrap(),
        );
        assert_ne!(prog1, prog2);
        assert!(!prog1.syntax_eq(&prog2));

        let (prog1, prog2) = (parse("abort").unwrap(), parse("abort \"\"").unwrap());
        assert_ne!(prog1, prog2);
        assert!(!prog1.syntax_eq(&prog2));
    }

    #[test]
    fn equal_literal_value_keeps_syntactic_equality() {
        let (prog1, prog2) = (
            parse(r#"run("foo")"#).unwrap(),
            parse(r#" run("foo")"#).unwrap(),
        );
        assert_ne!(prog1, prog2);
        assert!(
            prog1.syntax_eq(&prog2),
            "prog1 = {prog1:#?}, prog2 = {prog2:#?}"
        );

        let (prog1, prog2) = (
            parse(r#"run("{{ interop }}-val")"#).unwrap(),
            parse(r#" run("{{ interop }}-val")"#).unwrap(),
        );
        assert_ne!(prog1, prog2);
        assert!(
            prog1.syntax_eq(&prog2),
            "prog1 = {prog1:#?}, prog2 = {prog2:#?}"
        );

        let (prog1, prog2) = (
            parse(r#"run("{{ interop }}-val")"#).unwrap(),
            parse(r#" run("{{ interop }}-other stuff")"#).unwrap(),
        );
        assert_ne!(prog1, prog2);
        assert!(
            !prog1.syntax_eq(&prog2),
            "prog1 = {prog1:#?}, prog2 = {prog2:#?}"
        );

        let (prog1, prog2) = (
            parse(r#"run("{{ interop }}-val")"#).unwrap(),
            parse(r#" run("{{ other }}-val")"#).unwrap(),
        );
        assert_ne!(prog1, prog2);
        assert!(
            !prog1.syntax_eq(&prog2),
            "prog1 = {prog1:#?}, prog2 = {prog2:#?}"
        );
    }
}
