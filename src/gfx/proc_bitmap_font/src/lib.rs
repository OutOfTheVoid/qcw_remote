use proc_macro::TokenStream;
use syn::parse_macro_input;

mod bitmap_font;

#[proc_macro]
pub fn bitmap_glyph(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    bitmap_font::bitmap_glyph_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
