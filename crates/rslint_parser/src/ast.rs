//! AST definitions for converting untyped syntax nodes into typed AST nodes.
//!
//! Every field of every AST node is optional, this is to allow the parser to recover
//! from any error and produce an ast from any source code. If you don't want to account for
//! optionals for everything, you can use ...

#[macro_use]
mod expr_ext;
mod generated;
mod stmt_ext;
mod ts_ext;

use crate::{syntax_node::*, util::SyntaxNodeExt, SyntaxKind, SyntaxText, TextRange};
use std::marker::PhantomData;

pub use self::{
	expr_ext::*,
	generated::{nodes::*, tokens::*},
	stmt_ext::*,
	ts_ext::*,
};

/// The main trait to go from untyped `SyntaxNode`  to a typed ast. The
/// conversion itself has zero runtime cost: ast and syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
pub trait AstNode {
	fn can_cast(kind: SyntaxKind) -> bool
	where
		Self: Sized;

	fn cast(syntax: SyntaxNode) -> Option<Self>
	where
		Self: Sized;

	fn syntax(&self) -> &SyntaxNode;

	fn text(&self) -> std::string::String {
		self.syntax().trimmed_text().to_string()
	}

	fn range(&self) -> TextRange {
		self.syntax().trimmed_range()
	}
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
	fn can_cast(token: SyntaxKind) -> bool
	where
		Self: Sized;

	fn cast(syntax: SyntaxToken) -> Option<Self>
	where
		Self: Sized;

	fn syntax(&self) -> &SyntaxToken;

	fn text(&self) -> &str {
		self.syntax().text()
	}
}

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
	inner: SyntaxNodeChildren,
	ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
	fn new(parent: &SyntaxNode) -> Self {
		AstChildren {
			inner: parent.children(),
			ph: PhantomData,
		}
	}

	fn new_from_children(children: SyntaxNodeChildren) -> Self {
		AstChildren {
			inner: children,
			ph: PhantomData,
		}
	}
}

impl<N: AstNode> Iterator for AstChildren<N> {
	type Item = N;
	fn next(&mut self) -> Option<N> {
		self.inner.find_map(N::cast)
	}
}

#[derive(Debug, Clone)]
pub struct AstNodeList<N> {
	inner: SyntaxList,
	ph: PhantomData<N>,
}

impl<N: AstNode> AstNodeList<N> {
	fn new(parent: &SyntaxNode) -> Self {
		AstNodeList {
			inner: SyntaxList::new(parent.clone()),
			ph: PhantomData,
		}
	}

	pub fn iter(&self) -> AstChildren<N> {
		AstChildren::new_from_children(self.inner.iter())
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.inner.len()
	}

	#[inline]
	pub fn first(&self) -> Option<N> {
		// TODO 1724: Use inner once trivia is attached to tokens (not safe yet)
		self.iter().next()
	}

	pub fn last(&self) -> Option<N> {
		// TODO 1724: Use inner once trivia is attached to tokens (not safe yet)
		self.iter().last()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.inner.is_empty()
	}

	#[inline]
	pub fn text(&self) -> SyntaxText {
		self.inner.text()
	}

	pub fn text_range(&self) -> TextRange {
		self.inner.text_range()
	}

	pub fn parent(&self) -> Option<SyntaxNode> {
		self.inner.parent().map(SyntaxNode::from)
	}
}

impl<N: AstNode> IntoIterator for AstNodeList<N> {
	type Item = N;
	type IntoIter = AstChildren<N>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

mod support {
	use super::{AstNode, AstNodeList, SyntaxKind, SyntaxNode, SyntaxToken};
	use crate::ast::AstChildren;

	pub(super) fn child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
		parent.children().find_map(N::cast)
	}

	pub(super) fn children<N: AstNode>(parent: &SyntaxNode) -> AstChildren<N> {
		AstChildren::new(parent)
	}

	pub(super) fn list<N: AstNode>(parent: &SyntaxNode) -> AstNodeList<N> {
		// It's a parser or mutation error if a list isn't present in a many-child (field: T*).
		let list = parent
			.children()
			.find(|e| e.kind() == SyntaxKind::LIST)
			.expect("Expected a node list.");

		AstNodeList::new(&list)
	}

	pub(super) fn token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
		parent
			.children_with_tokens()
			.filter_map(|it| it.into_token())
			.find(|it| it.kind() == kind)
	}
}
