use proc_macro::TokenStream;
use quote::quote;
use rand::Rng;
use syn::{parse_macro_input, ItemFn};

/// A macro to obfuscate the control flow of a function by injecting opaque predicates.
#[proc_macro_attribute]
pub fn obfuscate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;

    let mut rng = rand::thread_rng();

    // Generate a random opaque predicate:
    // We know that for any integer x, x * x + x is always even, so (x*x+x) % 2 == 0 is ALWAYS true.
    let random_var_name = format!("_opaque_{}", rng.gen::<u32>());
    let random_var = syn::Ident::new(&random_var_name, proc_macro2::Span::call_site());

    // We create an opaque predicate condition
    // let `random_var` = rand() % 1000;
    // if (`random_var` * `random_var` + `random_var`) % 2 == 0 { ... } else { panic!("Cheat detected") }

    let obfuscated_block = quote! {
        {
            let #random_var: u64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() % 100;
            if (#random_var.wrapping_mul(#random_var).wrapping_add(#random_var)) % 2 == 0 {
                #fn_block
            } else {
                // This branch is mathematically unreachable, but static analysis tools don't know that.
                // We add some junk code here to confuse decompilers.
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    if #random_var == 99999 { break; }
                }
                panic!("Obfuscation error");
            }
        }
    };

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig #obfuscated_block
    };

    TokenStream::from(expanded)
}

//
