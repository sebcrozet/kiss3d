extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Macro to simplify writing cross-platform (native + WASM) kiss3d applications.
///
/// This macro wraps an async main function and generates the appropriate
/// platform-specific entry points:
/// - On native platforms: uses `pollster::block_on`
/// - On WASM: uses `wasm_bindgen_futures::spawn_local`
///
/// # Example
///
/// ```rust
/// #[kiss3d::main]
/// async fn main() {
///     let mut window = Window::new("My App");
///     while window.render_async().await {
///         // Your render loop
///     }
/// }
/// ```
///
/// This expands to platform-specific code that handles the async runtime
/// appropriately for each target.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Extract function components
    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let body = &input.block;

    // Verify the function signature
    if sig.ident != "main" {
        return syn::Error::new_spanned(
            &sig.ident,
            "the #[kiss3d::main] attribute can only be applied to a function named 'main'",
        )
        .to_compile_error()
        .into();
    }

    if sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            sig,
            "the #[kiss3d::main] attribute requires an async function",
        )
        .to_compile_error()
        .into();
    }

    if !sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &sig.inputs,
            "the main function cannot have parameters",
        )
        .to_compile_error()
        .into();
    }

    // Generate the expanded code
    let result = quote! {
        #[cfg(not(target_arch = "wasm32"))]
        #(#attrs)*
        #vis fn main() {
            pollster::block_on(__kiss3d_async_main())
        }

        #[cfg(target_arch = "wasm32")]
        #(#attrs)*
        #vis fn main() {
            wasm_bindgen_futures::spawn_local(__kiss3d_async_main())
        }

        async fn __kiss3d_async_main() #body
    };

    TokenStream::from(result)
}
