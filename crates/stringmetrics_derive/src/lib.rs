// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, DeriveInput};

// /// We expect something like this for usage:
// ///
// /// ```
// /// #[derive(AffixToken)]
// /// #[affix_key = "MYKEY"]
// /// #[affix_format = "bool"]
// /// ```
// ///
// #[proc_macro_derive(AffixToken, attributes(affix_key, affix_format))]
// pub fn affix_macro(input: TokenStream) -> TokenStream {
//     // Parse the input tokens into a syntax tree
//     // panic!("{:?}", input);
//     let input = parse_macro_input!(input as DeriveInput);
//     // panic!("{:?}",input);

//     // Build the output, possibly using quasi-quotation
//     let expanded = impl_affix_bool(input);

//     // Hand the output tokens back to the compiler
//     TokenStream::from(expanded)
// }

// fn impl_affix_bool(ast: DeriveInput) -> TokenStream {
//     let name = &ast.ident;
//     match &ast.data {
//         syn::Data::Struct(_) => (),
//         _ => panic!("Only allowed on structs!"),
//     }

//     let attr0 = ast.attrs.get(0).expect("Seems to be missing proper keys");
//     let attr1 = ast.attrs.get(1).expect("Seems to be missing proper keys");

//     let path0: String = attr0.path.get_ident().expect("Missing path 0").to_string();
//     let path1: String = attr1.path.get_ident().expect("Missing path 1").to_string();
//     if (path0 != "affix_key") || (path1 != "affix_format") {
//         panic!(
//             "Must define \"affix_key\" and \"affix_format\" in that order. Got: \"{}\" \"{}\"",
//             path0, path1
//         )
//     }

//     let token0: String = attr0.tokens.to_string();
//     let affix_key = token0.strip_prefix("= \"").expect("Bad format").strip_suffix("\"").expect("Bad format");

//     let token1: String = attr1.tokens.to_string();
//     let affix_format = token1.strip_prefix("= \"").expect("Bad format").strip_suffix("\"").expect("Bad format");

//     panic!("{:#?} and {:#?}\n", affix_key, affix_format);

//     quote! {
//         impl AffixToken for #name {
//             fn load_str(&mut self, s: &str) -> () {
//                 let a = "key here";
//                 // let mut b = stringify!(#a1);
//                 // let c = stringify!(#pt);
//                 // let d = stringify!(#st);
//                 // let e = stringify!(#bt);
//                 ()

//             }
//             fn update_parent(&self) -> (){()}
//         }
//     }
//     .into()
// }
