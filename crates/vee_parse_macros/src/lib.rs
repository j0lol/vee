use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, ItemStruct, Lit, LitInt, Meta, parse_macro_input};

/// For internal use. Annotates a bitfield so I don't have to do a bunch of boilerplate.
/// Enables special binrw maps if n ∈ `[8, 16, 32, 64]`,
/// otherwise you may not use `arbitrary_int` values in your bitfield.
#[proc_macro_attribute]
pub fn bitfield(attr: TokenStream, item: TokenStream) -> TokenStream {
    let bit_size = parse_macro_input!(attr as LitInt)
        .base10_parse::<u32>()
        .unwrap_or(32);

    let bit_type = match bit_size {
        8 => Some(quote! { u8 }),
        16 => Some(quote! { u16 }),
        32 => Some(quote! { u32 }),
        64 => Some(quote! { u64 }),
        _ => None,
    };

    let binrw_attrs = if let Some(bit_type) = bit_type {
        quote! {
            #[derive(BinRead, BinWrite)]
            #[br(map = #bit_type::into)]
            #[bw(map = |&x| #bit_type::from(x))]
        }
    } else {
        quote! {}
    };

    // Parse the struct
    let input = parse_macro_input!(item as ItemStruct);

    // Extract fields and generate layout doc string
    let mut layout_lines = Vec::new();
    layout_lines.push("struct BitField {".to_string());
    for field in input.fields.iter() {
        let docs = &field.attrs;
        // let docs = quote! {#(#docs)*}.to_string();

        for doc in docs {
            if let Meta::NameValue(meta) = &doc.meta {
                let path = &meta.path;
                if quote! {#path}.to_string() == "doc" {
                    let value = &meta.value;
                    let Expr::Lit(value) = value else { panic!() };
                    let Lit::Str(lit) = &value.lit else { panic!() };
                    let mut tok = lit.token().to_string();
                    tok.replace_range(0..2, "");
                    tok.replace_range((tok.len() - 1)..tok.len(), "");
                    let tok = tok.replace("\\", "");
                    layout_lines.push(format!("    /// {tok}"));
                }
            }
        }

        let ident = &field.ident;
        let ident = quote! {#ident}.to_string();

        let ty = &field.ty;
        let ty = quote! {#ty}.to_string();

        layout_lines.push(format!("    {ident}: {ty},"));
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
        #[doc = "```text"]
        #[doc = #layout_doc]
        #[doc = "```"]
        #[bitsize(#bit_size)]
        #[derive(FromBits, DebugBits, PartialEq, Clone, Copy)]
        #binrw_attrs
        #vis struct #struct_name #fields
    };

    TokenStream::from(expanded)
}
