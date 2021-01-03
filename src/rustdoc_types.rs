#[repr(u8)]
#[derive(
    Debug, Clone, Copy, serde_repr::Deserialize_repr, serde_repr::Serialize_repr, PartialEq, Eq,
)]
// Should match `rustdoc::html::item_type::ItemType`
pub enum ItemType {
    Mod,
    ExternCrate,
    Import,
    Struct,
    Enum,
    Fn,
    Type,
    Static,
    Trait,
    Impl,
    Tymethod,
    Method,
    Structfield,
    Variant,
    Macro,
    Primitive,
    AssociatedType,
    Constant,
    AssociatedConstant,
    Union,
    ForeignType,
    Keyword,
    Existential,
    Attr,
    Derive,
    TraitAlias,
}

impl ItemType {
    pub const fn to_url_slug(self) -> &'static str {
        match self {
            ItemType::Mod => "mod",
            ItemType::ExternCrate => "externcrate",
            ItemType::Import => "import",
            ItemType::Struct => "struct",
            ItemType::Enum => "enum",
            ItemType::Fn => "fn",
            ItemType::Type => "type",
            ItemType::Static => "static",
            ItemType::Trait => "trait",
            ItemType::Impl => "impl",
            ItemType::Tymethod => "tymethod",
            ItemType::Method => "method",
            ItemType::Structfield => "structfield",
            ItemType::Variant => "variant",
            ItemType::Macro => "macro",
            ItemType::Primitive => "primitive",
            ItemType::AssociatedType => "associatedtype",
            ItemType::Constant => "constant",
            ItemType::AssociatedConstant => "associatedconstant",
            ItemType::Union => "union",
            ItemType::ForeignType => "foreigntype",
            ItemType::Keyword => "keyword",
            ItemType::Existential => "existential",
            ItemType::Attr => "attr",
            ItemType::Derive => "derive",
            ItemType::TraitAlias => "traitalias",
        }
    }
}

#[derive(Debug, Clone, serde_tuple::Deserialize_tuple)]
pub struct Item {
    pub ty: ItemType,
    pub name: String,
    pub path: String,
    pub desc: String,
    pub parent: Option<usize>,
    pub wtf: serde_json::Value,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Path {
    pub ty: ItemType,
    pub name: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SearchIndex {
    pub doc: String,
    #[serde(rename = "i")]
    pub items: Vec<Item>,
    #[serde(rename = "p")]
    pub paths: Vec<Path>,
}
