use proc_macro::TokenStream;
use proc_macro_error::*;
use syn;
use quote::quote;

struct Flag {
    value: TokenStream,
    name: String,
    method_name: Option<syn::Ident>,
    doc: Option<String>,
}

impl Flag {
    fn method_name(&self) -> syn::Ident {
	if let Some(ident) = &self.method_name {
	    ident.clone()
	} else {
	    syn::Ident::new(&self.name, proc_macro2::Span::call_site())
	}
    }

}

enum DebugMode {
    None,
    Standard,
    Compact
}

struct FlagImpl {
    struct_name: syn::Ident,
    backing_field_name: syn::Member,
    flags: Vec<Flag>,
    debug_mode: DebugMode
}

fn get_value(lit: &syn::Lit, value_type: &syn::Type) -> TokenStream {
    use syn::Lit::*;
    
    let result = match lit {
	Int(_) => quote!{
	    #lit as #value_type
	},
	Str(s) => {
	    let expr: syn::Expr = match syn::parse_str(&s.value()) {
		Ok(expr) => expr,
		_ => {
		    abort!(lit, "String must contain a valid expression");
	        }
	    };
	    quote! {
		(#expr) as #value_type
	    }
	},
	_ => abort!(lit, "Bad value, must be an integer literal or string.")
    };
    result.into()
}

fn get_name(lit: &syn::Lit) -> String {
    use syn::Lit::*;

    match &lit {
	Str(s) => s.value(),
	_ => panic!("Bad name")
    }
}

fn get_method_name(lit: &syn::Lit) -> syn::Ident {
    use syn::{Lit::*, Ident};

    match &lit {
	Str(s) => Ident::new(&s.value(), lit.span()),
	_ => panic!("Bad method_name")
    }
}

fn get_doc(lit: &syn::Lit) -> String {
    use syn::Lit::*;

    match &lit {
	Str(s) => s.value(),
	_ => panic!("Bad doc attribute")
    }
}

fn parse_flag(attr: syn::Meta, value_type: &syn::Type) -> Flag {
    let mut name = None;
    let mut value = None;
    let mut method_name = None;
    let mut doc = None;
    
    if let syn::Meta::List(attr) = &attr {
	use syn::{Meta::NameValue, NestedMeta::Meta};
	let args = &attr.nested;
	
	for arg in args {
	    if let Meta(NameValue(m)) = arg {
	    	if let Some(n) = m.path.get_ident() {
		    match n.to_string().as_str() {
			"name" => name = Some(get_name(&m.lit)),
			"value" => value = Some(get_value(&m.lit, value_type)),
			"method_name" => method_name = Some(get_method_name(&m.lit)),
			"doc" => doc = Some(get_doc(&m.lit)),
			s => abort!(arg, r#"Unknown configuration option "{}". Expected one of [name, value, method_name, doc]"#, s)
		    }
		}
	    }
	}
    }

    if let (Some(name), Some(value)) = (name, value) {
	Flag {
	    name,
	    value,
	    method_name,
	    doc
	}
    } else {
	abort!(attr, "Missing name or value argument for flag.")
    }
}

fn get_backing_field(input: &syn::DeriveInput) -> (syn::Member, syn::Field) {
    let st = if let syn::Data::Struct(ds) = &input.data {
	ds
    } else {
	abort!(input, "Must be a struct")
    };

    let candidates: Vec<(syn::Member, &syn::Field)> = match &st.fields {
	syn::Fields::Named(named) => {
	    named.named.iter()
		.filter(|f| f.attrs.iter().any(|a| a.path.is_ident("flag_backing_field")))
		.map(|f| (syn::Member::Named(f.ident.clone().unwrap()), f))
		.collect()
	},
	syn::Fields::Unnamed(unnamed) => {
	    unnamed.unnamed.iter()
		.enumerate()
		.filter(|(_, f)| f.attrs.iter().any(|a| a.path.is_ident("flag_backing_field")))
		.map(|(i, f)| (syn::Member::Unnamed(syn::Index::from(i)), f))
		.collect()
	},
	_ => vec![]
    };

    if candidates.len() == 1 {
	let (ident, field) = candidates.first().unwrap();
	(ident.clone().into(), (*field).clone().into())
    } else {
	abort!(input, r#"Exactly one backing field must have the "flag_backing_field" attribute"#)
    }
}

fn parse_impl(input: TokenStream) -> FlagImpl {
    use syn::Meta::*;
    
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let (backing_field_name, backing_field) = get_backing_field(&ast);
    let struct_name = ast.ident.clone();
    let mut flags = vec![];
    let mut debug_mode = DebugMode::None;
    
    for attr in ast.attrs {
	if let Some(name) = attr.path.get_ident() {
	    match name.to_string().as_str() {
		"flag" => {
		    let meta = attr.parse_meta().unwrap_or_else(|_| abort!(attr, "Bad attribute arguments"));
		    let flag = parse_flag(meta, &backing_field.ty);
		    flags.push(flag);
		},
		"flag_debug" => {
		    let meta = attr.parse_meta();
		    match meta {
			Ok(Path(_)) => debug_mode = DebugMode::Standard,
			Ok(List(ml)) => {
			    if let Some(syn::NestedMeta::Meta(m)) = ml.nested.first() {
				if ml.nested.len() == 1 && m.path().is_ident("compact") {
				    debug_mode = DebugMode::Compact;
				    continue;
				} else {
				    abort!(ml, "Bad option for flag_meta attribute");
				}
			    } else {
				debug_mode = DebugMode::Standard;
			    }
			}
			_ => abort!(attr, "Bad attribute arguments")
		    }
		}
		_ => ()
	    }
	}
    }

    FlagImpl {
	struct_name,
	backing_field_name,
	flags,
	debug_mode
    }
}

#[proc_macro_derive(Flags, attributes(flag, flag_backing_field, flag_debug))]
#[proc_macro_error]
pub fn derive_flags(input: TokenStream) -> TokenStream {
    let mut flag_impl = parse_impl(input);
    let backing_field_name = flag_impl.backing_field_name;
    let struct_name = flag_impl.struct_name;

    let mut methods = vec![];

    let mut debug_fragments = vec![];

    for flag in flag_impl.flags.drain(..) {
	use quote::format_ident;
	let name = flag.name.clone();
	let method_name = flag.method_name();
	let value: proc_macro2::TokenStream = flag.value.into();
	
	match flag_impl.debug_mode {
	    DebugMode::None => (),
	    DebugMode::Standard => {
		debug_fragments.push(quote!{
		    .field(stringify!(#method_name), &self.#method_name())
		});
	    },
	    DebugMode::Compact => {
		debug_fragments.push(quote!{
		    if self.#method_name() {
			dbg.entry(&#name);
		    }
		});
	    }
	}

	let (doc, set_doc, only_doc) = {
	    let doc_template = "Gets the value for the flag.";
	    let set_template = "Sets the flag to the given value.";
	    let only_template = "Checks if this flag is the only one set.";
	    
	    if let Some(doc) = flag.doc {
		let doc_str = format!("{}\n\n{}", doc, doc_template);
		let set_str = format!("{}\n\n{}", doc, set_template);
		let only_str = format!("{}\n\n{}", doc, only_template);
		(
		    quote!{
			#[doc = #doc_str]
		    },
		    quote!{
			#[doc = #set_str]
		    },
		    quote!{
			#[doc = #only_str]
		    }
		)
	    } else {
		(
		    quote!{
			#[doc = #doc_template]
		    },
		    quote!{
			#[doc = #set_template]
		    },
		    quote!{
			#[doc = #only_template]
		    }
		)
	    }
	};

	let setter_name = format_ident!("set_{}", method_name);
	let exclusive_name = format_ident!("only_{}", method_name);
	let flag_methods = quote!{
	    #doc
	    pub fn #method_name(&self) -> bool {
		self.#backing_field_name & (#value) == (#value)
	    }
	    #only_doc
	    pub fn #exclusive_name(&self) -> bool {
		self.#backing_field_name | (#value) == (#value)
	    }
	    #set_doc
	    pub fn #setter_name(&mut self, value: bool) -> &Self {
		if value {
		    self.#backing_field_name |= (#value);
		} else {
		    self.#backing_field_name &= !(#value)
		}
		self
	    }
	};
	
	methods.push(flag_methods);
	
    }

    let main_impl = quote!{
	impl #struct_name {
	    #(#methods)*
	}
    };

    let debug_impl = match flag_impl.debug_mode {
	DebugMode::None => quote!{},
	DebugMode::Standard => quote!{
	    impl core::fmt::Debug for #struct_name {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		    f.debug_struct(stringify!(#struct_name))
			#(#debug_fragments)*
		    .finish()
		}
	    }
	},
	DebugMode::Compact => quote!{
	    impl core::fmt::Debug for #struct_name {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		    write!(f, "{} ", stringify!(#struct_name))?;
		    let mut dbg = f.debug_set();
		    #(#debug_fragments)*
		    dbg.finish()
		}
	    }
	}
    };

    (quote!{
	#main_impl

	#debug_impl
    }).into()
}
