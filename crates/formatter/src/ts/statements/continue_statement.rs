use crate::{
	empty_element, format_elements, space_token, token, FormatElement, Formatter, ToFormatElement,
};
use rslint_parser::ast::ContinueStmt;

impl ToFormatElement for ContinueStmt {
	fn to_format_element(&self, formatter: &Formatter) -> Option<FormatElement> {
		// NOTE: rslint parser (upstream) is currently broken https://github.com/rslint/rslint/issues/126
		let ident = if let Some(ident_token) = self.ident_token() {
			format_elements![space_token(), formatter.format_node(ident_token)?]
		} else {
			empty_element()
		};
		let continue_token = formatter.format_token(&self.continue_token()?)?;
		Some(format_elements![continue_token, ident, token(";")])
	}
}
