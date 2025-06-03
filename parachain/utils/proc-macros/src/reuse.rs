// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{
	parse::Parse, punctuated::Punctuated, visit_mut::VisitMut, Arm, Attribute, Block, ConstParam,
	Error, Expr, ExprArray, ExprAssign, ExprAsync, ExprAwait, ExprBinary, ExprBlock, ExprBreak,
	ExprCall, ExprCast, ExprClosure, ExprConst, ExprContinue, ExprField, ExprForLoop, ExprGroup,
	ExprIf, ExprIndex, ExprInfer, ExprLet, ExprLit, ExprLoop, ExprMacro, ExprMatch, ExprMethodCall,
	ExprParen, ExprPath, ExprRange, ExprReference, ExprRepeat, ExprReturn, ExprStruct, ExprTry,
	ExprTryBlock, ExprTuple, ExprUnary, ExprUnsafe, ExprWhile, ExprYield, Field, FieldPat,
	FieldValue, FnArg, GenericParam, Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn,
	ItemForeignMod, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait,
	ItemTraitAlias, ItemType, ItemUnion, ItemUse, LifetimeParam, Local, Meta, Pat, PatConst,
	PatIdent, PatLit, PatMacro, PatOr, PatParen, PatPath, PatRange, PatReference, PatRest,
	PatSlice, PatStruct, PatTuple, PatTupleStruct, PatType, PatWild, Receiver, Result, Stmt,
	StmtMacro, Token, TypeParam, Variant,
};

pub struct Args {
	cases: Punctuated<Ident, Token![,]>,
}

impl Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> Result<Self> {
		Ok(Self { cases: Punctuated::parse_terminated(input)? })
	}
}

pub fn handle_reuse(args: Args, m: ItemMod) -> Result<TokenStream> {
	let mut result = TokenStream::new();
	let cases = args.cases.iter().map(ToString::to_string).collect::<Vec<_>>();
	for current in 0..args.cases.len() {
		let mut visitor = Visitor { cases: &cases, current, error: None };
		let mut m = m.clone();
		m.ident = args.cases[current].clone();
		visitor.visit_item_mod_mut(&mut m);
		if let Some(e) = visitor.error.take() {
			return Err(e);
		}
		m.to_tokens(&mut result);
	}
	Ok(result)
}

struct Visitor<'a> {
	cases: &'a [String],
	current: usize,
	error: Option<Error>,
}

impl VisitMut for Visitor<'_> {
	fn visit_item_mod_mut(&mut self, m: &mut ItemMod) {
		if let Some((_, items)) = &mut m.content {
			let mut i = 0;
			while i < items.len() {
				if let Some(attrs) = items[i].get_attrs_mut() {
					match self.case(attrs) {
						Case::Other => {
							items.remove(i);
							continue;
						},
						Case::Current => {
							if let [Item::Fn(this), Item::Fn(next), ..] = &mut items[i..] {
								if let [Stmt::Expr(
									Expr::Path(ExprPath { attrs, qself: None, path }),
									None,
								)] = this.block.stmts.as_slice()
								{
									if attrs.is_empty() && path.is_ident("__") {
										this.block.clone_from(&next.block);
									}
								}
							}
						},
						_ => (),
					}
				}
				i += 1;
			}
		}
		syn::visit_mut::visit_item_mod_mut(self, m)
	}

	fn visit_block_mut(&mut self, block: &mut Block) {
		self.filter_vec(&mut block.stmts);
		syn::visit_mut::visit_block_mut(self, block)
	}

	fn visit_signature_mut(&mut self, item: &mut syn::Signature) {
		self.filter_punct(&mut item.inputs);
		syn::visit_mut::visit_signature_mut(self, item)
	}

	fn visit_generics_mut(&mut self, item: &mut syn::Generics) {
		self.filter_punct(&mut item.params);
		syn::visit_mut::visit_generics_mut(self, item)
	}

	fn visit_expr_tuple_mut(&mut self, item: &mut ExprTuple) {
		self.filter_punct(&mut item.elems);
		syn::visit_mut::visit_expr_tuple_mut(self, item)
	}

	fn visit_pat_tuple_mut(&mut self, item: &mut syn::PatTuple) {
		self.filter_punct(&mut item.elems);
		syn::visit_mut::visit_pat_tuple_mut(self, item)
	}

	fn visit_pat_tuple_struct_mut(&mut self, item: &mut PatTupleStruct) {
		self.filter_punct(&mut item.elems);
		syn::visit_mut::visit_pat_tuple_struct_mut(self, item)
	}

	fn visit_pat_struct_mut(&mut self, item: &mut PatStruct) {
		self.filter_punct(&mut item.fields);
		syn::visit_mut::visit_pat_struct_mut(self, item)
	}

	fn visit_expr_struct_mut(&mut self, item: &mut ExprStruct) {
		self.filter_punct(&mut item.fields);
		syn::visit_mut::visit_expr_struct_mut(self, item)
	}

	fn visit_fields_named_mut(&mut self, item: &mut syn::FieldsNamed) {
		self.filter_punct(&mut item.named);
		syn::visit_mut::visit_fields_named_mut(self, item)
	}

	fn visit_fields_unnamed_mut(&mut self, item: &mut syn::FieldsUnnamed) {
		self.filter_punct(&mut item.unnamed);
		syn::visit_mut::visit_fields_unnamed_mut(self, item)
	}

	fn visit_item_enum_mut(&mut self, item: &mut ItemEnum) {
		self.filter_punct(&mut item.variants);
		syn::visit_mut::visit_item_enum_mut(self, item)
	}

	fn visit_item_union_mut(&mut self, item: &mut ItemUnion) {
		self.filter_punct(&mut item.fields.named);
		syn::visit_mut::visit_item_union_mut(self, item)
	}

	fn visit_expr_match_mut(&mut self, item: &mut ExprMatch) {
		self.filter_vec(&mut item.arms);
		syn::visit_mut::visit_expr_match_mut(self, item)
	}

	fn visit_pat_or_mut(&mut self, item: &mut PatOr) {
		self.filter_punct(&mut item.cases);
		syn::visit_mut::visit_pat_or_mut(self, item)
	}

	fn visit_expr_closure_mut(&mut self, item: &mut ExprClosure) {
		self.filter_punct(&mut item.inputs);
		syn::visit_mut::visit_expr_closure_mut(self, item)
	}
}

enum Case {
	Current,
	Other,
	All,
}

impl Visitor<'_> {
	#[inline]
	fn filter_vec<T>(&self, target: &mut Vec<T>)
	where
		T: GetAttrsMut,
	{
		let mut i = 0;
		while i < target.len() {
			if let Some(attrs) = target[i].get_attrs_mut() {
				if let Case::Other = self.case(attrs) {
					target.remove(i);
					continue;
				}
			}
			i += 1;
		}
	}

	#[inline]
	fn filter_punct<T, P>(&self, target: &mut Punctuated<T, P>)
	where
		T: GetAttrsMut + Clone,
		P: Default,
	{
		let mut copy = None;
		for i in 0..target.len() {
			if let Some(attrs) = target[i].get_attrs_mut() {
				if let Case::Other = self.case(attrs) {
					copy.get_or_insert_with(|| {
						Punctuated::from_iter(target.iter().take(i).cloned())
					});
				} else if let Some(copy) = &mut copy {
					copy.push(target[i].clone());
				}
			}
		}
		if let Some(copy) = copy {
			*target = copy;
		}
	}

	#[inline]
	fn case(&self, attrs: &mut Vec<Attribute>) -> Case {
		let mut is_casing = false;
		let mut has_current = false;
		let mut i = 0;
		while i < attrs.len() {
			if let Meta::Path(path) = &attrs[i].meta {
				if let Some(ident) = path.get_ident() {
					if ident == self.cases[self.current].as_str() {
						has_current = true;
						is_casing = true;
						attrs.remove(i);
						continue;
					}
					if self.cases.iter().any(|x| ident == x.as_str()) {
						is_casing = true;
						attrs.remove(i);
						continue;
					}
				}
			}
			i += 1;
		}
		if has_current {
			Case::Current
		} else if is_casing {
			Case::Other
		} else {
			Case::All
		}
	}
}

trait GetAttrsMut {
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>;
}

impl GetAttrsMut for Item {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		match self {
			Item::Const(ItemConst { attrs, .. })
			| Item::Enum(ItemEnum { attrs, .. })
			| Item::ExternCrate(ItemExternCrate { attrs, .. })
			| Item::Fn(ItemFn { attrs, .. })
			| Item::ForeignMod(ItemForeignMod { attrs, .. })
			| Item::Impl(ItemImpl { attrs, .. })
			| Item::Macro(ItemMacro { attrs, .. })
			| Item::Mod(ItemMod { attrs, .. })
			| Item::Static(ItemStatic { attrs, .. })
			| Item::Struct(ItemStruct { attrs, .. })
			| Item::Trait(ItemTrait { attrs, .. })
			| Item::TraitAlias(ItemTraitAlias { attrs, .. })
			| Item::Type(ItemType { attrs, .. })
			| Item::Union(ItemUnion { attrs, .. })
			| Item::Use(ItemUse { attrs, .. }) => Some(attrs),
			_ => None,
		}
	}
}

impl GetAttrsMut for Expr {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		match self {
			Expr::Array(ExprArray { attrs, .. })
			| Expr::Assign(ExprAssign { attrs, .. })
			| Expr::Async(ExprAsync { attrs, .. })
			| Expr::Await(ExprAwait { attrs, .. })
			| Expr::Binary(ExprBinary { attrs, .. })
			| Expr::Block(ExprBlock { attrs, .. })
			| Expr::Break(ExprBreak { attrs, .. })
			| Expr::Call(ExprCall { attrs, .. })
			| Expr::Cast(ExprCast { attrs, .. })
			| Expr::Closure(ExprClosure { attrs, .. })
			| Expr::Const(ExprConst { attrs, .. })
			| Expr::Continue(ExprContinue { attrs, .. })
			| Expr::Field(ExprField { attrs, .. })
			| Expr::ForLoop(ExprForLoop { attrs, .. })
			| Expr::Group(ExprGroup { attrs, .. })
			| Expr::If(ExprIf { attrs, .. })
			| Expr::Index(ExprIndex { attrs, .. })
			| Expr::Infer(ExprInfer { attrs, .. })
			| Expr::Let(ExprLet { attrs, .. })
			| Expr::Lit(ExprLit { attrs, .. })
			| Expr::Loop(ExprLoop { attrs, .. })
			| Expr::Macro(ExprMacro { attrs, .. })
			| Expr::Match(ExprMatch { attrs, .. })
			| Expr::MethodCall(ExprMethodCall { attrs, .. })
			| Expr::Paren(ExprParen { attrs, .. })
			| Expr::Path(ExprPath { attrs, .. })
			| Expr::Range(ExprRange { attrs, .. })
			| Expr::Reference(ExprReference { attrs, .. })
			| Expr::Repeat(ExprRepeat { attrs, .. })
			| Expr::Return(ExprReturn { attrs, .. })
			| Expr::Struct(ExprStruct { attrs, .. })
			| Expr::Try(ExprTry { attrs, .. })
			| Expr::TryBlock(ExprTryBlock { attrs, .. })
			| Expr::Tuple(ExprTuple { attrs, .. })
			| Expr::Unary(ExprUnary { attrs, .. })
			| Expr::Unsafe(ExprUnsafe { attrs, .. })
			| Expr::While(ExprWhile { attrs, .. })
			| Expr::Yield(ExprYield { attrs, .. }) => Some(attrs),
			_ => None,
		}
	}
}

impl GetAttrsMut for Stmt {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		match self {
			Stmt::Local(Local { attrs, .. }) | Stmt::Macro(StmtMacro { attrs, .. }) => Some(attrs),
			Stmt::Expr(expr, ..) => expr.get_attrs_mut(),
			Stmt::Item(item) => item.get_attrs_mut(),
		}
	}
}

impl GetAttrsMut for FnArg {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		match self {
			FnArg::Receiver(Receiver { attrs, .. }) | FnArg::Typed(PatType { attrs, .. }) => {
				Some(attrs)
			},
		}
	}
}

impl GetAttrsMut for GenericParam {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		match self {
			GenericParam::Lifetime(LifetimeParam { attrs, .. })
			| GenericParam::Type(TypeParam { attrs, .. })
			| GenericParam::Const(ConstParam { attrs, .. }) => Some(attrs),
		}
	}
}

impl GetAttrsMut for Pat {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		match self {
			Pat::Const(PatConst { attrs, .. })
			| Pat::Ident(PatIdent { attrs, .. })
			| Pat::Lit(PatLit { attrs, .. })
			| Pat::Macro(PatMacro { attrs, .. })
			| Pat::Or(PatOr { attrs, .. })
			| Pat::Paren(PatParen { attrs, .. })
			| Pat::Path(PatPath { attrs, .. })
			| Pat::Range(PatRange { attrs, .. })
			| Pat::Reference(PatReference { attrs, .. })
			| Pat::Rest(PatRest { attrs, .. })
			| Pat::Slice(PatSlice { attrs, .. })
			| Pat::Struct(PatStruct { attrs, .. })
			| Pat::Tuple(PatTuple { attrs, .. })
			| Pat::TupleStruct(PatTupleStruct { attrs, .. })
			| Pat::Type(PatType { attrs, .. })
			| Pat::Wild(PatWild { attrs, .. }) => Some(attrs),
			_ => None,
		}
	}
}

impl GetAttrsMut for FieldPat {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		Some(&mut self.attrs)
	}
}

impl GetAttrsMut for FieldValue {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		Some(&mut self.attrs)
	}
}

impl GetAttrsMut for Field {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		Some(&mut self.attrs)
	}
}

impl GetAttrsMut for Variant {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		Some(&mut self.attrs)
	}
}

impl GetAttrsMut for Arm {
	#[inline]
	fn get_attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		Some(&mut self.attrs)
	}
}
