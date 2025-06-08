use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, LitInt, parse_macro_input};

/// For internal use. Annotates a bitfield so I don't have to do a bunch of boilerplate.
/// Defaults to 32 if you don't pass it an integer n ∈ `[8, 16, 32, 64]`
#[proc_macro_attribute]
pub fn bitfield(attr: TokenStream, item: TokenStream) -> TokenStream {
    let bit_size = parse_macro_input!(attr as LitInt)
        .base10_parse::<u32>()
        .unwrap_or(32);

    let bit_type = match bit_size {
        8 => quote! { u8 },
        16 => quote! { u16 },
        32 => quote! { u32 },
        64 => quote! { u64 },
        _ => quote! { u32 },
    };

    // Parse the struct
    let input = parse_macro_input!(item as ItemStruct);

    // Extract fields and generate layout doc string
    let mut layout_lines = Vec::new();
    layout_lines.push("struct BitField {".to_string());
    for field in input.fields.iter() {
        let ident = &field.ident;
        let ident = quote! {#ident}.to_string();

        let ty = &field.ty;
        let ty = quote! {#ty}.to_string();

        layout_lines.push(format!("    {ident}: {ty}"));
    }
    layout_lines.push("}".to_string());

    let layout_doc = layout_lines.join("\n");

    let struct_name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let fields = &input.fields;

    let expanded = quote! {
        #(#attrs)*
        #[doc = "A bit field."]
        #[doc = "```rs"]
        #[doc = #layout_doc]
        #[doc = "```"]
        #[bitsize(#bit_size)]
        #[derive(FromBits, DebugBits, BinRead, BinWrite, PartialEq, Clone, Copy)]
        #[br(map = #bit_type::into)]
        #[bw(map = |&x| #bit_type::from(x))]
        #vis struct #struct_name #fields
    };

    TokenStream::from(expanded)
}
