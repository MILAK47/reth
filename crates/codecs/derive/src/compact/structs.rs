use super::*;

#[derive(Debug)]
pub struct StructHandler<'a> {
    fields_iterator: std::iter::Peekable<std::slice::Iter<'a, FieldTypes>>,
    lines: Vec<TokenStream2>,
}

impl<'a> StructHandler<'a> {
    pub fn new(fields: &'a FieldList) -> Self {
        StructHandler { lines: vec![], fields_iterator: fields.iter().peekable() }
    }

    pub fn next_field(&mut self) -> Option<&'a FieldTypes> {
        self.fields_iterator.next()
    }

    pub fn generate_to(mut self) -> Vec<TokenStream2> {
        while let Some(field) = self.next_field() {
            match field {
                //  The following method will advance the
                // `fields_iterator` by itself and stop right before the next variant.
                FieldTypes::EnumVariant(_) => unreachable!(),
                FieldTypes::EnumUnnamedField(_) => unreachable!(),
                FieldTypes::StructField(field_descriptor) => self.to(field_descriptor),
            }
        }
        self.lines
    }

    pub fn generate_from(mut self, known_types: &[&str]) -> Vec<TokenStream2> {
        while let Some(field) = self.next_field() {
            match field {
                //  The following method will advance the
                // `fields_iterator` by itself and stop right before the next variant.
                FieldTypes::EnumVariant(_) => unreachable!(),
                FieldTypes::EnumUnnamedField(_) => unreachable!(),
                FieldTypes::StructField(field_descriptor) => {
                    self.from(field_descriptor, known_types)
                }
            }
        }
        self.lines
    }

    /// Generates `to_compact` code for a struct field.
    fn to(&mut self, field_descriptor: &StructFieldDescriptor) {
        let (name, ftype, is_compact) = field_descriptor;

        let name = format_ident!("{name}");
        let set_len_method = format_ident!("set_{name}_len");
        let len = format_ident!("{name}_len");

        // H256 with #[maybe_zero] attribute for example
        if *is_compact && !is_flag_type(ftype) {
            let itype = format_ident!("{ftype}");
            let set_bool_method = format_ident!("set_{name}");
            self.lines.push(quote! {
                if self.#name != #itype::zero() {
                    flags.#set_bool_method(true);
                    self.#name.to_compact(&mut buffer);
                };
            });
        } else {
            self.lines.push(quote! {
                let #len = self.#name.to_compact(&mut buffer);
            });
        }
        if is_flag_type(ftype) {
            self.lines.push(quote! {
                flags.#set_len_method(#len as u8);
            })
        }
    }

    /// Generates `from_compact` code for a struct field.
    fn from(&mut self, field_descriptor: &StructFieldDescriptor, known_types: &[&str]) {
        let (name, ftype, is_compact) = field_descriptor;

        let name = format_ident!("{name}");
        let len = format_ident!("{name}_len");

        assert!(
            known_types.contains(&ftype.as_str()) ||
                is_flag_type(ftype) ||
                self.fields_iterator.peek().is_none(),
            "{ftype} field should be placed as the last one since it's not known. "
        );

        if ftype == "bytes::Bytes" {
            self.lines.push(quote! {
                let mut #name = bytes::Bytes::new();
                (#name, buf) = bytes::Bytes::from_compact(buf, buf.len() as usize);
            })
        } else {
            let ident_type = format_ident!("{ftype}");
            self.lines.push(quote! {
                let mut #name = #ident_type::default();
            });
            if !is_flag_type(ftype) {
                // It's a type that handles its own length requirements. (h256, Custom, ...)
                self.lines.push(quote! {
                    (#name, buf) = #ident_type::from_compact(buf, buf.len());
                })
            } else if *is_compact {
                self.lines.push(quote! {
                    (#name, buf) = #ident_type::from_compact(buf, flags.#len() as usize);
                });
            } else {
                todo!()
            }
        }
    }
}
